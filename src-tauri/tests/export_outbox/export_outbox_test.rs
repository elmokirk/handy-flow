//! KB-402 acceptance: exports are durable, retryable and idempotent.
//!
//! An export leaves the app and touches a filesystem someone else owns. That
//! makes it the one place where "we already did this" and "we crashed
//! halfway" must both be answerable from the database alone.
//!
//! Frozen state machine (orchestration/DATA_STATE_MACHINES.md):
//! `pending → running → succeeded | retry_wait | failed_permanent`,
//! plus `retry_wait → running`.

use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::exports::{
    claim_next, enqueue, jobs_for_export, mark_failed, mark_succeeded, requeue_stale_running,
    upsert_target, EnqueueOutcome, NewExportTarget, MAX_ATTEMPTS,
};

fn fresh(tag: &str) -> AppDatabase {
    let dir = std::env::temp_dir().join(format!("handy-kb402-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    db
}

fn target(db: &AppDatabase) -> String {
    upsert_target(
        db,
        &NewExportTarget {
            kind: "markdown".into(),
            root_path: "C:/vault".into(),
        },
    )
    .unwrap()
    .id
}

const NOW: i64 = 1_757_000_000_000;

/// 01 — the same item with the same content never queues twice. Without
/// this, every export sweep would rewrite the whole vault.
#[test]
fn enqueueing_the_same_content_twice_is_idempotent() {
    let db = fresh("01");
    let t = target(&db);

    let first = enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();
    let second = enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();

    let EnqueueOutcome::Enqueued(job) = first else {
        panic!("first enqueue must create a job");
    };
    let EnqueueOutcome::AlreadyQueued(existing) = second else {
        panic!("second enqueue must recognise the duplicate");
    };
    assert_eq!(job.id, existing.id);
    assert_eq!(jobs_for_export(&db, "capture:abc").unwrap().len(), 1);
}

/// 02 — changed content IS a new job. Idempotency must not become "we never
/// update anything".
#[test]
fn changed_content_queues_a_new_job() {
    let db = fresh("02");
    let t = target(&db);

    enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();
    let second = enqueue(&db, &t, "capture:abc", "hash-2", NOW).unwrap();

    assert!(matches!(second, EnqueueOutcome::Enqueued(_)));
    assert_eq!(jobs_for_export(&db, "capture:abc").unwrap().len(), 2);
}

/// 03 — claiming is atomic: a claimed job is `running` and no second worker
/// can claim it. Two writers exporting the same file would corrupt it.
#[test]
fn a_claimed_job_cannot_be_claimed_again() {
    let db = fresh("03");
    let t = target(&db);
    enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();

    let claimed = claim_next(&db, NOW)
        .unwrap()
        .expect("a pending job is claimable");
    assert_eq!(claimed.status, "running");

    assert!(
        claim_next(&db, NOW).unwrap().is_none(),
        "a running job must not be handed out twice"
    );
}

/// 04 — a failure becomes `retry_wait` with a future, growing backoff. The
/// job is never dropped and never retried in a tight loop.
#[test]
fn failure_moves_to_retry_wait_with_growing_backoff() {
    let db = fresh("04");
    let t = target(&db);
    enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();

    let job = claim_next(&db, NOW).unwrap().unwrap();
    let after_one = mark_failed(&db, &job.id, "vault offline", NOW).unwrap();
    assert_eq!(after_one.status, "retry_wait");
    assert_eq!(after_one.attempt_count, 1);
    assert_eq!(after_one.last_error.as_deref(), Some("vault offline"));
    let first_delay = after_one.next_attempt_at_ms.unwrap() - NOW;
    assert!(first_delay > 0, "a retry must be scheduled in the future");

    let job = claim_next(&db, NOW + first_delay).unwrap().unwrap();
    let after_two = mark_failed(&db, &job.id, "vault offline", NOW + first_delay).unwrap();
    assert_eq!(after_two.attempt_count, 2);
    let second_delay = after_two.next_attempt_at_ms.unwrap() - (NOW + first_delay);
    assert!(
        second_delay > first_delay,
        "backoff must grow: {first_delay} -> {second_delay}"
    );
}

/// 05 — a job in `retry_wait` is invisible until its time has come.
#[test]
fn retry_wait_is_not_claimable_before_its_scheduled_time() {
    let db = fresh("05");
    let t = target(&db);
    enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();
    let job = claim_next(&db, NOW).unwrap().unwrap();
    let waiting = mark_failed(&db, &job.id, "offline", NOW).unwrap();
    let due = waiting.next_attempt_at_ms.unwrap();

    assert!(claim_next(&db, due - 1).unwrap().is_none(), "too early");
    assert!(claim_next(&db, due).unwrap().is_some(), "due now");
}

/// 06 — after the attempt budget the job becomes `failed_permanent` and
/// STAYS. No silent loss: a permanently failed export is still queryable.
#[test]
fn exhausted_retries_end_in_failed_permanent_and_remain_visible() {
    let db = fresh("06");
    let t = target(&db);
    enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();

    let mut now = NOW;
    let mut last = None;
    for _ in 0..MAX_ATTEMPTS {
        let job = claim_next(&db, now).unwrap().expect("still retryable");
        let failed = mark_failed(&db, &job.id, "disk full", now).unwrap();
        now = failed.next_attempt_at_ms.unwrap_or(now + 1);
        last = Some(failed);
    }

    let last = last.unwrap();
    assert_eq!(last.status, "failed_permanent");
    assert_eq!(last.attempt_count, MAX_ATTEMPTS);
    assert!(
        claim_next(&db, now + 1_000_000).unwrap().is_none(),
        "a permanently failed job must not be retried forever"
    );
    let jobs = jobs_for_export(&db, "capture:abc").unwrap();
    assert_eq!(jobs.len(), 1, "the job is kept, not deleted");
    assert_eq!(jobs[0].status, "failed_permanent");
}

/// 07 — the crash case. A job left `running` when the app died must come
/// back, or the export is silently lost.
#[test]
fn a_job_stranded_in_running_is_requeued() {
    let db = fresh("07");
    let t = target(&db);
    enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();
    claim_next(&db, NOW).unwrap().unwrap(); // ...and the process dies here.

    let requeued = requeue_stale_running(&db, NOW + 60_000).unwrap();

    assert_eq!(requeued, 1);
    let job = claim_next(&db, NOW + 60_000)
        .unwrap()
        .expect("the stranded job must be claimable again");
    assert_eq!(job.attempt_count, 0, "a crash is not the job's fault");
}

/// 08 — a succeeded export is not redone. Re-enqueueing identical content
/// reports it as already exported instead of queueing a second write.
#[test]
fn succeeded_content_is_not_exported_again() {
    let db = fresh("08");
    let t = target(&db);
    enqueue(&db, &t, "capture:abc", "hash-1", NOW).unwrap();
    let job = claim_next(&db, NOW).unwrap().unwrap();
    mark_succeeded(&db, &job.id, NOW + 5).unwrap();

    let again = enqueue(&db, &t, "capture:abc", "hash-1", NOW + 10).unwrap();

    let EnqueueOutcome::AlreadyExported(done) = again else {
        panic!("identical content must not be exported twice, got {again:?}");
    };
    assert_eq!(done.status, "succeeded");
    assert!(done.completed_at_ms.is_some());
    assert_eq!(jobs_for_export(&db, "capture:abc").unwrap().len(), 1);
}

/// 09 — two targets are independent. The same item exported to two vaults
/// is two jobs, not a deduplicated one.
#[test]
fn the_same_item_can_be_exported_to_two_targets() {
    let db = fresh("09");
    let a = target(&db);
    let b = upsert_target(
        &db,
        &NewExportTarget {
            kind: "markdown".into(),
            root_path: "D:/second-brain".into(),
        },
    )
    .unwrap()
    .id;
    assert_ne!(a, b);

    enqueue(&db, &a, "capture:abc", "hash-1", NOW).unwrap();
    let second = enqueue(&db, &b, "capture:abc", "hash-1", NOW).unwrap();

    assert!(matches!(second, EnqueueOutcome::Enqueued(_)));
    assert_eq!(jobs_for_export(&db, "capture:abc").unwrap().len(), 2);
}

/// 10 — a target is identified by what it points at, so re-registering the
/// same vault does not fork the outbox.
#[test]
fn upserting_the_same_target_returns_the_same_id() {
    let db = fresh("10");

    let first = target(&db);
    let second = target(&db);

    assert_eq!(first, second);
}

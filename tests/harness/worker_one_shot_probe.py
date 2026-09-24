"""Read-only one-shot worker probe; never prints or stores transcript text."""

import argparse
from collections import Counter
import hashlib
import json
import re
import sqlite3
import subprocess
import threading
import time
import wave
from pathlib import Path


def request(worker, payload):
    worker.stdin.write(json.dumps(payload) + "\n")
    worker.stdin.flush()
    response = worker.stdout.readline()
    if not response:
        raise RuntimeError("worker exited without a response")
    return json.loads(response)


def checksum(path):
    with path.open("rb") as audio:
        return hashlib.file_digest(audio, "sha256").hexdigest()


def trigrams(text):
    words = re.findall(r"\w+", text.casefold())
    return Counter(zip(words, words[1:], words[2:]))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("executable", type=Path)
    parser.add_argument("model_id")
    parser.add_argument("audio", type=Path)
    parser.add_argument("--compare-db", type=Path)
    parser.add_argument("--ceiling-seconds", type=int, default=360)
    args = parser.parse_args()

    with wave.open(str(args.audio), "rb") as audio:
        assert (audio.getnchannels(), audio.getframerate(), audio.getsampwidth()) == (
            1, 16_000, 2
        )
        samples = audio.getnframes()
    before = checksum(args.audio)
    worker = subprocess.Popen(
        [str(args.executable), "--transcription-worker"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        bufsize=1,
    )
    try:
        loaded = request(worker, {"command": "load", "model_id": args.model_id})
        if loaded.get("error"):
            raise RuntimeError(f"model load failed: {loaded['error']}")
        limit = loaded.get("max_audio_samples")
        if limit is None or samples > min(limit, args.ceiling_seconds * 16_000):
            raise RuntimeError(f"one-shot not eligible: samples={samples}, model_limit={limit}")
        started = time.monotonic()
        watchdog = threading.Timer(90, worker.kill)
        watchdog.start()
        try:
            result = request(
                worker,
                {"command": "transcribe", "path": str(args.audio), "start": 0, "end": samples},
            )
        finally:
            watchdog.cancel()
        if result.get("error"):
            raise RuntimeError(f"transcription failed: {result['error']}")
        raw = result["output"]["engine_raw"]
        print(
            f"samples={samples} model_limit={limit} "
            f"seconds={time.monotonic() - started:.3f} raw_chars={len(raw)}"
        )
        if args.compare_db:
            with sqlite3.connect(args.compare_db) as db:
                old = db.execute(
                    "SELECT a.engine_raw FROM captures c JOIN transcription_attempts a ON a.capture_id=c.id WHERE c.audio_file_name=? AND a.is_canonical=1",
                    (args.audio.name,),
                ).fetchone()[0]
            current, previous = trigrams(raw), trigrams(old)
            overlap = sum((current & previous).values())
            print(
                f"trigram_coverage: one_shot={overlap / max(sum(current.values()), 1):.3f} "
                f"existing={overlap / max(sum(previous.values()), 1):.3f}"
            )
    finally:
        if worker.poll() is None:
            worker.terminate()
        worker.wait(timeout=10)
        assert checksum(args.audio) == before, "original WAV changed"


if __name__ == "__main__":
    main()

//! Connector adapter skeleton.
//!
//! Thin read-only transport adapters over the QueryService domain:
//! - REST companion process (loopback-only, bearer auth)
//! - MCP stdio server
//!
//! Forbidden by contract: raw SQL access, direct repository calls,
//! writes of any kind. Phase 5 (QUERY-501/REST/MCP tickets) fills this
//! module — intentionally empty here.

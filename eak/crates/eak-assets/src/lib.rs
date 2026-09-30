//! Asset acquisition and management for EAK component library.
//!
//! This crate provides:
//! - Asset type definitions (datasheet, symbol, footprint, 3D model, app note)
//! - Asset state machine with honest blocking behavior
//! - File validation (PDF signature, KiCad S-expr, STEP header)
//! - SHA-256 deduplication
//! - Provenance tracking
//! - Manual import path
//! - Source matrix for acquisition strategies
//! - HTTP downloader with honest error handling
//! - Automated acquisition pipeline
//! - Datasheet parsing and knowledge extraction

pub mod acquisition;
pub mod asset;
pub mod crosscheck;
pub mod downloader;
pub mod facts;
pub mod identity;
pub mod import;
pub mod parser;
pub mod provenance;
pub mod source;
pub mod store;
pub mod validation;

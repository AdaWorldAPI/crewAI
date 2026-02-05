//! RAG (Retrieval-Augmented Generation) framework for crewAI.
//!
//! This module provides the building blocks for RAG pipelines:
//!
//! - **core** - Base traits for loaders, chunkers, and embedding services
//! - **loaders** - Concrete document loaders for various file formats and sources
//! - **chunkers** - Text chunking strategies for splitting documents into segments

pub mod chunkers;
pub mod core;
pub mod loaders;

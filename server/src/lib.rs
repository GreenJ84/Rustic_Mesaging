#![allow(unused)]
#![recursion_limit = "256"]

#[macro_use]
extern crate rocket;

pub(crate) mod api_error;
pub mod app;
pub(crate) mod config;
pub(crate) mod database;
pub (crate) mod cors;
pub(crate) mod id_generator;
pub(crate) mod schema;
pub(crate) mod utils;

pub(crate) mod auth;
pub(crate) mod chat;
pub(crate) mod member;
pub(crate) mod server;
pub(crate) mod social;
pub(crate) mod audit;

#[cfg(test)]
pub(crate) mod tests;

use rocket::response::status;
pub(crate) type ApiResponse<T> = Result<status::Custom<T>, api_error::ApiError>;

pub(in crate::social) mod models;
pub(in crate::social) mod repository;

#[cfg(test)]
#[path = "../../tests/social/request/repository.rs"]
mod repository_integration_tests;

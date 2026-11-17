pub(in crate::social) mod models;
pub(crate) mod repository;

#[cfg(test)]
#[path = "../../tests/social/friend/repository.rs"]
mod repository_integration_tests;

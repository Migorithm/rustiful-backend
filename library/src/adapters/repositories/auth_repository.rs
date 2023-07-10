
use crate::domain::auth::AuthAggregate;

use super::{Repository};
use crate::utils::ApplicationError;



impl Repository<AuthAggregate> {
 

    async fn _add(&mut self, _aggregate: &AuthAggregate) -> Result<String, ApplicationError> {
        unimplemented!()
    }

    async fn get(&self, _aggregate_id: &str) -> Result<AuthAggregate, ApplicationError> {
        unimplemented!()
    }

    async fn _update(&mut self, _aggregate: &AuthAggregate) -> Result<(), ApplicationError> {
        unimplemented!()
    }
}

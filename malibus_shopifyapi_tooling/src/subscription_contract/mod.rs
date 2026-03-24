#![allow(non_snake_case)]
pub mod getSubscriptionContract;
pub mod subscriptionContractDraftCreate;
pub mod subscriptionDraftUpdate;
pub mod commitSubscriptionDraft;
pub mod subscriptionDraftLineUpdate;
pub mod subscriptionDraftLineAdd;
pub mod subscriptionDraftLineRemove;
        
// used for pre-purchasing (futures) - otherwise just update a current contract
pub mod subscriptionContractCreate; 

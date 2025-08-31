use crate::{ address::Address, are_addresses_equal };
use std::collections::HashMap;

// Wrapped string as role
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct NamedRole(String);

pub enum UnauthorizedError {
    AddressNotAuthorized,
}

pub struct Authorizer {
    pub role_assignees: HashMap<NamedRole, Address>,
    pub is_role_known: HashMap<NamedRole, bool>,
    role_manager_address: Address,
    
    // implement role group later
}

impl Authorizer {
    pub fn new(role_manager_address: Address) -> Authorizer {
        Authorizer {
            is_role_known: HashMap::new(),
            role_assignees: HashMap::new(),
            role_manager_address,
        }
    }

    pub fn change_role_manager_address(
        &mut self,
        new_address: Address,
        caller_address: Address
    ) -> Result<(), String> {
        if caller_address != self.role_manager_address {
            return Err("only role manager is allowed to transfer role manager ownership".into());
        }

        self.role_manager_address = new_address;
        Ok(())
    }

    pub fn only_authorized_role(
        &self,
        allowed_roles: &[NamedRole],
        caller_addres: Address
    ) -> Result<(), UnauthorizedError> {
        for role in allowed_roles {
            let role_assignee = self.role_assignees.get(role);
            if (
                match role_assignee {
                    Some(role_assignee) => are_addresses_equal(role_assignee, &caller_addres),
                    None => false,
                }
            ) {
                return Ok(());
            }
        }

        return Err(UnauthorizedError::AddressNotAuthorized);
    }

    pub fn make_role_known(
        &mut self,
        new_role: NamedRole,
        caller_addres: Address
    ) -> Result<(), UnauthorizedError> {
        // only role manager
        self.only_authorized_address(&[&self.role_manager_address], caller_addres)?;
        self.is_role_known.insert(new_role, true);
        Ok(())
    }
}

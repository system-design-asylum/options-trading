use crate::{address::Address, are_addresses_equal};
use std::collections::HashMap;

// Wrapped string as role
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct NamedRole(pub String);

#[derive(Debug)]
pub enum UnauthorizedError {
    AddressNotAuthorized,
}

pub struct RoleAuthorizer {
    pub role_assignees: HashMap<NamedRole, Address>,
    pub is_role_known: HashMap<NamedRole, bool>,
    // TODO: implement role groups later
}

// TODO: refactor in later ver
pub fn placeholder_role_manager_address() -> Address {
    Address::from("0xb73B0A92544a5D2523F00F868d795d50DbDfcCf4")
        .expect("Invalid placeholder role manager address literal")
}

impl RoleAuthorizer {
    pub fn new(role_manager_addr: Address) -> RoleAuthorizer {
        let mut authorizer = RoleAuthorizer {
            is_role_known: HashMap::new(),
            role_assignees: HashMap::new(),
        };

        // The role that manages addition, removal, assignment of other roles
        let role = NamedRole("RolesManager".to_string());

        authorizer.is_role_known.insert(role.clone(), true);
        authorizer
            .role_assignees
            .insert(role.clone(), role_manager_addr);
        return authorizer;
    }

    pub fn change_role_manager_address(
        &mut self,
        new_address: Address,
        caller_address: Address,
    ) -> Result<(), UnauthorizedError> {
        let role = NamedRole("RolesManager".to_string());
        self.only_authorized_role(&[role.clone()], caller_address)?;
        self.role_assignees.insert(role.clone(), new_address);
        Ok(())
    }

    pub fn only_authorized_role(
        &self,
        allowed_roles: &[NamedRole],
        caller_addres: Address,
    ) -> Result<(), UnauthorizedError> {
        for role in allowed_roles {
            let role_assignee = self.role_assignees.get(role);
            if (match role_assignee {
                Some(role_assignee) => are_addresses_equal(role_assignee, &caller_addres),
                None => false,
            }) {
                return Ok(());
            }
        }

        return Err(UnauthorizedError::AddressNotAuthorized);
    }

    pub fn make_role_known(
        &mut self,
        new_role: NamedRole,
        caller_addres: Address,
    ) -> Result<(), UnauthorizedError> {
        // only role manager can perform
        let role = NamedRole("RolesManager".to_string());
        self.only_authorized_role(&[role], caller_addres)?;
        self.is_role_known.insert(new_role, true);
        Ok(())
    }

    pub fn assign_role(
        &mut self,
        role: NamedRole,
        assignee: Address,
        caller_address: Address,
    ) -> Result<(), UnauthorizedError> {
        let roles_manager = NamedRole("RolesManager".to_string());
        self.only_authorized_role(&[roles_manager], caller_address)?;
        self.role_assignees.insert(role, assignee);
        Ok(())
    }
}

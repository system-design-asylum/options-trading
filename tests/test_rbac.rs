use options_trading::{Address, rbac::{NamedRole, RoleAuthorizer, UnauthorizedError}};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_address(suffix: &str) -> Address {
        let base = "0x123456789012345678901234567890123456789";
        let full_address = format!("{}{}", base, suffix);
        // Ensure address is exactly 42 characters by padding/truncating
        let padded = format!("{:0<42}", full_address);
        Address::from(&padded[..42]).unwrap()
    }

    #[test]
    fn test_role_authorizer_creation() {
        let manager_addr = create_test_address("1");
        let authorizer = RoleAuthorizer::new(manager_addr.clone());

        // Should have RolesManager role assigned to the manager
        let roles_manager_role = NamedRole("RolesManager".to_string());
        let result = authorizer.only_authorized_role(&[roles_manager_role], manager_addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_only_authorized_role_success() {
        let manager_addr = create_test_address("1");
        let authorizer = RoleAuthorizer::new(manager_addr.clone());

        let roles_manager_role = NamedRole("RolesManager".to_string());
        let result = authorizer.only_authorized_role(&[roles_manager_role], manager_addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_only_authorized_role_unauthorized() {
        let manager_addr = create_test_address("1");
        let unauthorized_addr = create_test_address("2");
        let authorizer = RoleAuthorizer::new(manager_addr);

        let roles_manager_role = NamedRole("RolesManager".to_string());
        let result = authorizer.only_authorized_role(&[roles_manager_role], unauthorized_addr);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            UnauthorizedError::AddressNotAuthorized => {}
        }
    }

    #[test]
    fn test_make_role_known_success() {
        let manager_addr = create_test_address("1");
        let mut authorizer = RoleAuthorizer::new(manager_addr.clone());

        let new_role = NamedRole("Admin".to_string());
        let result = authorizer.make_role_known(new_role.clone(), manager_addr.clone());
        assert!(result.is_ok());

        // Role should be known but not assigned to anyone yet
        let result = authorizer.only_authorized_role(&[new_role], manager_addr);
        assert!(result.is_err()); // Should fail because no one is assigned this role yet
    }

    #[test]
    fn test_make_role_known_unauthorized() {
        let manager_addr = create_test_address("1");
        let unauthorized_addr = create_test_address("2");
        let mut authorizer = RoleAuthorizer::new(manager_addr);

        let new_role = NamedRole("Admin".to_string());
        let result = authorizer.make_role_known(new_role, unauthorized_addr);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            UnauthorizedError::AddressNotAuthorized => {}
        }
    }

    #[test]
    fn test_assign_role_success() {
        let manager_addr = create_test_address("1");
        let admin_addr = create_test_address("2");
        let mut authorizer = RoleAuthorizer::new(manager_addr.clone());

        // First make the role known
        let admin_role = NamedRole("Admin".to_string());
        authorizer.make_role_known(admin_role.clone(), manager_addr.clone()).unwrap();

        // Then assign it
        let result = authorizer.assign_role(admin_role.clone(), admin_addr.clone(), manager_addr);
        assert!(result.is_ok());

        // Now the admin should be authorized
        let result = authorizer.only_authorized_role(&[admin_role], admin_addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assign_role_unauthorized() {
        let manager_addr = create_test_address("1");
        let admin_addr = create_test_address("2");
        let unauthorized_addr = create_test_address("3");
        let mut authorizer = RoleAuthorizer::new(manager_addr.clone());

        let admin_role = NamedRole("Admin".to_string());
        authorizer.make_role_known(admin_role.clone(), manager_addr).unwrap();

        // Try to assign role as unauthorized user
        let result = authorizer.assign_role(admin_role, admin_addr, unauthorized_addr);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            UnauthorizedError::AddressNotAuthorized => {}
        }
    }

    #[test]
    fn test_change_role_manager_address_success() {
        let old_manager_addr = create_test_address("1");
        let new_manager_addr = create_test_address("2");
        let mut authorizer = RoleAuthorizer::new(old_manager_addr.clone());

        let result = authorizer.change_role_manager_address(new_manager_addr.clone(), old_manager_addr.clone());
        assert!(result.is_ok());

        // Old manager should no longer be authorized
        let roles_manager_role = NamedRole("RolesManager".to_string());
        let result = authorizer.only_authorized_role(&[roles_manager_role.clone()], old_manager_addr);
        assert!(result.is_err());

        // New manager should be authorized
        let result = authorizer.only_authorized_role(&[roles_manager_role], new_manager_addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_change_role_manager_address_unauthorized() {
        let manager_addr = create_test_address("1");
        let new_manager_addr = create_test_address("2");
        let unauthorized_addr = create_test_address("3");
        let mut authorizer = RoleAuthorizer::new(manager_addr.clone());

        let result = authorizer.change_role_manager_address(new_manager_addr, unauthorized_addr);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            UnauthorizedError::AddressNotAuthorized => {}
        }

        // Original manager should still be authorized
        let roles_manager_role = NamedRole("RolesManager".to_string());
        let result = authorizer.only_authorized_role(&[roles_manager_role], manager_addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_roles_authorization() {
        let manager_addr = create_test_address("1");
        let user_addr = create_test_address("2");
        let mut authorizer = RoleAuthorizer::new(manager_addr.clone());

        // Create multiple roles
        let admin_role = NamedRole("Admin".to_string());
        let moderator_role = NamedRole("Moderator".to_string());
        
        authorizer.make_role_known(admin_role.clone(), manager_addr.clone()).unwrap();
        authorizer.make_role_known(moderator_role.clone(), manager_addr.clone()).unwrap();

        // Assign only admin role to user
        authorizer.assign_role(admin_role.clone(), user_addr.clone(), manager_addr).unwrap();

        // User should be authorized for admin role
        let result = authorizer.only_authorized_role(&[admin_role.clone()], user_addr.clone());
        assert!(result.is_ok());

        // User should be authorized when checking for either admin OR moderator
        let result = authorizer.only_authorized_role(&[admin_role, moderator_role.clone()], user_addr.clone());
        assert!(result.is_ok());

        // User should NOT be authorized for moderator role only
        let result = authorizer.only_authorized_role(&[moderator_role], user_addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_role_reassignment() {
        let manager_addr = create_test_address("1");
        let first_admin_addr = create_test_address("2");
        let second_admin_addr = create_test_address("3");
        let mut authorizer = RoleAuthorizer::new(manager_addr.clone());

        let admin_role = NamedRole("Admin".to_string());
        authorizer.make_role_known(admin_role.clone(), manager_addr.clone()).unwrap();

        // Assign role to first admin
        authorizer.assign_role(admin_role.clone(), first_admin_addr.clone(), manager_addr.clone()).unwrap();

        // Verify first admin is authorized
        let result = authorizer.only_authorized_role(&[admin_role.clone()], first_admin_addr.clone());
        assert!(result.is_ok());

        // Reassign role to second admin
        authorizer.assign_role(admin_role.clone(), second_admin_addr.clone(), manager_addr).unwrap();

        // First admin should no longer be authorized (role was reassigned, not duplicated)
        let result = authorizer.only_authorized_role(&[admin_role.clone()], first_admin_addr);
        assert!(result.is_err());

        // Second admin should be authorized
        let result = authorizer.only_authorized_role(&[admin_role], second_admin_addr);
        assert!(result.is_ok());
    }
}

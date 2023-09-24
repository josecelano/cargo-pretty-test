use cargo_pretty_test::prettify::make_pretty;
use pretty_assertions::assert_eq;

#[test]
fn golden_master_test() {
    // Snapshot test for output after one generation

    const INPUT: &str = "
    test e2e::web::api::v1::contexts::category::contract::it_should_not_allow_adding_duplicated_categories ... ok
    test e2e::web::api::v1::contexts::tag::contract::it_should_not_allow_adding_duplicated_tags ... ok
    test e2e::web::api::v1::contexts::category::contract::it_should_not_allow_non_admins_to_delete_categories ... ok
    test e2e::web::api::v1::contexts::tag::contract::it_should_not_allow_adding_a_tag_with_an_empty_name ... ok
    test e2e::web::api::v1::contexts::tag::contract::it_should_not_allow_guests_to_delete_tags ... ok
    test e2e::web::api::v1::contexts::category::contract::it_should_allow_admins_to_delete_categories ... ok
    test e2e::web::api::v1::contexts::user::contract::banned_user_list::it_should_allow_an_admin_to_ban_a_user ... ok
    test e2e::web::api::v1::contexts::tag::contract::it_should_allow_admins_to_delete_tags ... fail
    test e2e::web::api::v1::contexts::user::contract::banned_user_list::it_should_not_allow_a_non_admin_to_ban_a_user ... ok
    test e2e::web::api::v1::contexts::tag::contract::it_should_not_allow_non_admins_to_delete_tags ... ok
    ";

    const OUTPUT: &str = "\
test
└── e2e
    └── web
        └── api
            └── v1
                └── contexts
                    ├── category
                    │   └── contract
                    │       ├─ ✅ it_should_allow_admins_to_delete_categories
                    │       ├─ ✅ it_should_not_allow_adding_duplicated_categories
                    │       └─ ✅ it_should_not_allow_non_admins_to_delete_categories
                    ├── tag
                    │   └── contract
                    │       ├─ ❌ it_should_allow_admins_to_delete_tags
                    │       ├─ ✅ it_should_not_allow_adding_a_tag_with_an_empty_name
                    │       ├─ ✅ it_should_not_allow_adding_duplicated_tags
                    │       ├─ ✅ it_should_not_allow_guests_to_delete_tags
                    │       └─ ✅ it_should_not_allow_non_admins_to_delete_tags
                    └── user
                        └── contract
                            └── banned_user_list
                                ├─ ✅ it_should_allow_an_admin_to_ban_a_user
                                └─ ✅ it_should_not_allow_a_non_admin_to_ban_a_user
";

    assert_eq!(
        make_pretty("test", INPUT.trim().lines().map(str::trim))
            .unwrap()
            .to_string(),
        OUTPUT
    );
}

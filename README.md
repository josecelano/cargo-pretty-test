# Cargo Pretty Test ✨

[![Testing](https://github.com/josecelano/pretty-test/actions/workflows/testing.yaml/badge.svg)](https://github.com/josecelano/pretty-test/actions/workflows/testing.yaml)

A Rust command that prettifies the ugly `cargo test` output into a beautiful output.

Input:

```s
test e2e::web::api::v1::contexts::category::contract::it_should_not_allow_adding_duplicated_categories ... ok
test e2e::web::api::v1::contexts::tag::contract::it_should_not_allow_adding_duplicated_tags ... ok
test e2e::web::api::v1::contexts::category::contract::it_should_not_allow_non_admins_to_delete_categories ... ok
test e2e::web::api::v1::contexts::tag::contract::it_should_not_allow_adding_a_tag_with_an_empty_name ... ok
test e2e::web::api::v1::contexts::tag::contract::it_should_not_allow_guests_to_delete_tags ... ok
test e2e::web::api::v1::contexts::category::contract::it_should_allow_admins_to_delete_categories ... ok
test e2e::web::api::v1::contexts::user::contract::banned_user_list::it_should_allow_an_admin_to_ban_a_user ... ok
test e2e::web::api::v1::contexts::tag::contract::it_should_allow_admins_to_delete_tags ... FAILED
test e2e::web::api::v1::contexts::user::contract::banned_user_list::it_should_not_allow_a_non_admin_to_ban_a_user ... ok
test e2e::web::api::v1::contexts::tag::contract::it_should_not_allow_non_admins_to_delete_tags ... ok
```

Output:

```s
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
```

## Usage

Install:

```console
cargo install cargo-pretty-test
```

Run in your project:

```console
cargo pretty-test
```

## Credits

- First commit author [@ZJPzjp](https://github.com/zjp-CN).
- Idea described on [https://users.rust-lang.org](https://users.rust-lang.org/t/cargo-test-output-with-indentation/100149) by [@josecelano](https://github.com/josecelano).

### Links

- <https://users.rust-lang.org/t/cargo-test-output-with-indentation/100149>
- <https://www.rustexplorer.com/b/i058g3>
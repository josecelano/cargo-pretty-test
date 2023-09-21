# Pretty-test ✨

[![Testing](https://github.com/josecelano/pretty-test/actions/workflows/testing.yaml/badge.svg)](https://github.com/josecelano/pretty-test/actions/workflows/testing.yaml)

A Rust command that prettifies the ugly `cargo test` into a beautiful output.

Input:

```s
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

## Run

```s
cat tests/fixtures/sample_cargo_test_output.txt | cargo run
```

You can also create a Rust script with <https://rust-script.org/>:

- `cargo install rust-script`.
- Execute the code.
- Save the code in a file named pretty-test.
- `chmod +x ./pretty-test`
- Add it in your environment, like `mv pretty-test ~/.cargo/bin`
- Run `pretty-test` in your project.

## Test

```s
cargo test
```

## Credits

- <https://users.rust-lang.org/t/cargo-test-output-with-indentation/100149>
- <https://www.rustexplorer.com/b/i058g3>

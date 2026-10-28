use diesel::sql_types::Text;

diesel::define_sql_function! {
    fn lower(value: Text) -> Text;
}

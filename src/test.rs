use shared::query::QueryParamValueType::Str;
use shared::route::Route;

#[cfg(test)]
pub fn route_query_parser_should_match_result(){
    // Arrange
    let path = "/hello?age:int&name?&amount:float&is_subscribed:bool&address:string?*".to_string();
    let keys = ["age", "name", "amount", "is_subscribed", "address"];

    // Act
    let (result, _) = Route::generate_queries(path);

    // Assert
    for key in keys {
        assert!(result.contains_key(key));
    }

    let name = result.get("name").unwrap();
    assert_eq!(name._type, Str(String::from("")));
    assert!(name.flags.is_optional);

    let address = result.get("address").unwrap();
    assert_eq!(address._type, Str(String::from("")));
    assert!(address.flags.is_array);
    assert!(address.flags.allow_empty);
}

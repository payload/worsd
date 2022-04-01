use std::error;

pub fn fetch_definition(word: &str) {
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
    match fetch_definition_unhandled(&url) {
        Ok(definitions) => println!("fetch_definition: {:?}", definitions),
        Err(err) => eprintln!("fetch_definition: {:?}", err),
    }
}

fn fetch_definition_unhandled(url: &str) -> Result<Vec<String>, Box<dyn error::Error>> {
    let response = minreq::get(url).send()?;
    let response_str = response.as_str()?;
    let response_json = json::parse(response_str)?;
    Ok(response_json[0]["meanings"]
        .members()
        .flat_map(|meaning| {
            meaning["definitions"]
                .members()
                .map(|def| &def["definition"])
        })
        .filter_map(|definition| definition.as_str())
        .map(String::from)
        .collect())
}

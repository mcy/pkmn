use pkmn::api::Cache;
use pkmn::model;
use pkmn::Api;

fn main() -> Result<(), pkmn::api::Error> {
  let mut api = Api::with_cache(Cache::new(256));

  for l in api.all::<model::Language>(50) {
    let l = l?;
    let en_name = l
      .localized_names
      .iter()
      .find(|t| t.language.is(model::text::LanguageName::English))
      .map(|t| &t.text);
    println!("{} {:?}", l.name, en_name);
  }

  Ok(())
}

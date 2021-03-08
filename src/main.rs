use pkmn::api::Cache;
use pkmn::model;
use pkmn::model::LanguageName;
use pkmn::Api;

fn main() -> Result<(), pkmn::api::Error> {
  let mut api = Api::with_cache(Cache::new(256));

  for l in api.all::<model::Language>(50) {
    let l = l?;
    println!(
      "{} {:?}",
      l.name,
      l.localized_names.get(LanguageName::English)
    );
  }

  Ok(())
}

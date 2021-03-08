use pkmn::api::Cache;
use pkmn::model;
use pkmn::Api;

fn main() -> Result<(), pkmn::api::Error> {
  let mut api = Api::with_cache(Cache::new(256));

  for l in api.all::<model::Language>(50) {
    println!("{}", l?.name);
  }

  Ok(())
}

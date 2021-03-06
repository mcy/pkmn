use pkmn::api::Cache;
use pkmn::model;
use pkmn::Api;

fn main() -> Result<(), pkmn::api::Error> {
  let mut api = Api::with_cache(Cache::no_disk(256));
  api.by_name::<model::mov::Move>("earth-power")?;

  Ok(())
}

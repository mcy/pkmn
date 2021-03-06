fn main() -> Result<(), pkmn::api::Error> {
  let mut api = pkmn::Api::with_cache(pkmn::api::Cache::no_disk(256));
  api.by_name::<pkmn::model::ability::Ability>("pressure")?;

  Ok(())
}

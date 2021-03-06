use pkmn::Api;
use pkmn::model::species::Species;

fn main() -> Result<(), pkmn::api::Error> {
  let mut api = Api::new();
  api.by_name::<Species>("mew")?;

  Ok(())
}

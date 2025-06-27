use menu;

pub struct Config {
    scales: Vec<ScaleConfig>,
}
pub enum ScaleType {
    Counter,
    Drawer,
}
pub struct ScaleConfig {
    scale_type: ScaleType,
    scale_sn: isize,
    phidget_sn: i32,
    load_cell_id: i32,
    gain: f64,
    offset: f64,
}

/*
fn read(path: &Path) -> Result<Self, Error> where Self: Sized, for<'de> Self: Deserialize<'de> {
        let file_as_string = fs::read_to_string(path).map_err(Error::FileSystem)?;
        toml::from_str(&file_as_string).map_err(Error::TomlRead)
    }
*/

use walkers::{sources::{Attribution, TileSource}, TileId};

pub struct MapaCarto;

impl TileSource for MapaCarto {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://d.basemaps.cartocdn.com/light_all/{}/{}/{}@2x.png",
            tile_id.zoom, tile_id.x, tile_id.y
        )
    }

    fn attribution(&self) -> Attribution { // Se debe llamar attribution
        Attribution {
            text: "Repo github",
            url: "https://github.com/taller-1-fiuba-rust/24C1-Los-Tallerines/tree/interfaz_grafica",
            logo_light: None,
            logo_dark: None,
        }
    }
}
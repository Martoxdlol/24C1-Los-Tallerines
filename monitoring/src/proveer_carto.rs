use walkers::{
    sources::{Attribution, TileSource},
    TileId,
};

pub struct MapaCarto;

impl TileSource for MapaCarto {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://d.basemaps.cartocdn.com/light_all/{}/{}/{}@2x.png",
            tile_id.zoom, tile_id.x, tile_id.y
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "CARTO Attribution",
            url: "https://carto.com/attribution",
            logo_light: None,
            logo_dark: None,
        }
    }
}

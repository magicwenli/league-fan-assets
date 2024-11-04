const COMMUNITY_DRAGON_URL: &str = "https://raw.communitydragon.org";
const DDRAGON_VERSIONS_URL: &str = "https://ddragon.leagueoflegends.com/api/versions.json";

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Config {
    pub version: Option<String>,
    pub language: LanguageType,
}

impl Config {
    pub fn default() -> Self {
        Self {
            version: None,
            language: LanguageType::default(),
        }
    }

    pub fn new(version: Option<String>, language: LanguageType) -> Self {
        Self { version, language }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum AssetsType {
    #[default]
    Loot,
    Skins,
    Skinlines,
    SummonerEmotes,
    SummonerBanners,
    SummonerIcons,
    SummonerIconSets,
    WardSkins,
    WardSkinSets,
}

impl AssetsType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetsType::Loot => "loot.json",
            AssetsType::Skins => "skins.json",
            AssetsType::Skinlines => "skinlines.json",
            AssetsType::SummonerEmotes => "summoner-emotes.json",
            AssetsType::SummonerBanners => "summoner-banners.json",
            AssetsType::SummonerIcons => "summoner-icons.json",
            AssetsType::SummonerIconSets => "summoner-icon-sets.json",
            AssetsType::WardSkins => "ward-skins.json",
            AssetsType::WardSkinSets => "ward-skin-sets.json",
        }
    }

    pub fn to_vec() -> Vec<AssetsType> {
        vec![
            AssetsType::Loot,
            AssetsType::Skins,
            AssetsType::Skinlines,
            AssetsType::SummonerEmotes,
            AssetsType::SummonerBanners,
            AssetsType::SummonerIcons,
            AssetsType::SummonerIconSets,
            AssetsType::WardSkins,
            AssetsType::WardSkinSets,
        ]
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LanguageType {
    #[default]
    Default,
    Arabic,
    Czech,
    German,
    Greek,
    EnglishAustralia,
    EnglishGreatBritain,
    EnglishPhilippines,
    EnglishSingapore,
    SpanishArgentina,
    SpanishSpain,
    SpanishMexico,
    FrenchFrance,
    Hungarian,
    Italian,
    Japanese,
    Korean,
    Polish,
    PortugueseBrazil,
    Romanian,
    Russian,
    Thai,
    Turkish,
    Vietnamese,
    ChineseChina,
    ChineseMalaysia,
    ChineseTaiwan,
}

impl LanguageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageType::Default => "default",
            LanguageType::Arabic => "ar_ae",
            LanguageType::Czech => "cs_cz",
            LanguageType::German => "de_de",
            LanguageType::Greek => "el_gr",
            LanguageType::EnglishAustralia => "en_au",
            LanguageType::EnglishGreatBritain => "en_gb",
            LanguageType::EnglishPhilippines => "en_ph",
            LanguageType::EnglishSingapore => "en_sg",
            LanguageType::SpanishArgentina => "es_ar",
            LanguageType::SpanishSpain => "es_es",
            LanguageType::SpanishMexico => "es_mx",
            LanguageType::FrenchFrance => "fr_fr",
            LanguageType::Hungarian => "hu_hu",
            LanguageType::Italian => "it_it",
            LanguageType::Japanese => "ja_jp",
            LanguageType::Korean => "ko_kr",
            LanguageType::Polish => "pl_pl",
            LanguageType::PortugueseBrazil => "pt_br",
            LanguageType::Romanian => "ro_ro",
            LanguageType::Russian => "ru_ru",
            LanguageType::Thai => "th_th",
            LanguageType::Turkish => "tr_tr",
            LanguageType::Vietnamese => "vi_vn",
            LanguageType::ChineseChina => "zh_cn",
            LanguageType::ChineseMalaysia => "zh_my",
            LanguageType::ChineseTaiwan => "zh_tw",
        }
    }
}

pub fn get_assets_url(
    assets_type: &AssetsType,
    language: &LanguageType,
    version: &Option<String>,
) -> String {
    match version {
        Some(version) => {
            // 14.21.1 ==> 14.21
            let version = version.split('.').take(2).collect::<Vec<&str>>().join(".");
            format!(
                "{}/{}/plugins/rcp-be-lol-game-data/global/{}/v1/{}",
                COMMUNITY_DRAGON_URL,
                version,
                language.as_str(),
                assets_type.as_str()
            )
        }
        None => format!(
            "{}/latest/plugins/rcp-be-lol-game-data/global/{}/v1/{}",
            COMMUNITY_DRAGON_URL,
            language.as_str(),
            assets_type.as_str()
        ),
    }
}

pub fn get_cdragon_url(ori_url: &str, config: &Config) -> String {
    // "/lol-game-data/assets/ASSETS/Loot/jhin_tile_37.jpg"
    // -> https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/loot/jhin_tile_37.jpg
    let trimed_url = ori_url.trim_start_matches("/lol-game-data/assets/");
    let version = match &config.version {
        Some(version) => version.split('.').take(2).collect::<Vec<&str>>().join("."),
        None => "latest".to_string(),
    };
    let mut url = format!(
        "{}/{}/plugins/rcp-be-lol-game-data/global/default/",
        COMMUNITY_DRAGON_URL, version,
    );
    url.push_str(trimed_url.to_ascii_lowercase().as_str());
    url
}

pub async fn get_game_versions() -> Result<Vec<String>, reqwest::Error> {
    reqwest::get(DDRAGON_VERSIONS_URL)
        .await?
        .json::<Vec<String>>()
        .await
}

pub async fn get_latest_version() -> Result<String, reqwest::Error> {
    let versions = get_game_versions().await?;
    Ok(versions[0].clone())
}

pub fn get_all_assets_urls(config: &Config) -> Vec<String> {
    let mut urls = Vec::new();
    for assets_type in AssetsType::to_vec().iter() {
        urls.push(get_assets_url(
            assets_type,
            &config.language,
            &config.version,
        ));
    }
    urls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_game_versions() {
        let versions = get_game_versions().await.unwrap();
        let target = "14.21.1";
        assert!(versions.contains(&target.to_string()));
    }
}

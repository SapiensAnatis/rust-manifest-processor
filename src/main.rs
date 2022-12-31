#![feature(hash_set_entry)]
use std::{fs, io::BufReader, error::Error, path::Path, collections::HashSet, hash::Hash};
use serde_derive::{Deserialize, Serialize};

const MANIFEST_PATH: &'static str = r#"D:\DragaliaLostAssets\DragaliaManifests-master\Android"#;

#[derive(Serialize, Deserialize)]
struct Asset {
    name: String,
    hash: String,
    size: i64,
}

impl Hash for Asset {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Asset {}

#[derive(Serialize, Deserialize)]
struct AssetCategory {
    name: String,
    assets: HashSet<Asset>,
    #[serde(alias = "encryptedAssets")]
    encrypted_assets: HashSet<Asset>,
}

impl AssetCategory {
    fn new(name: &str) -> AssetCategory {
        AssetCategory { name: String::from(name), assets: HashSet::new(), encrypted_assets: HashSet::new() }
    }
}

#[derive(Serialize, Deserialize)]
struct UnityHeader {
    #[serde(alias = "m_PathID")]
    path_id: i64,
    #[serde(alias = "m_FileID")]
    file_id: i32,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    #[serde(alias = "m_GameObject")]
    game_object: UnityHeader,
    #[serde(alias = "m_Enabled")]
    enabled: i32,
    #[serde(alias = "m_Script")]
    script: UnityHeader,
    #[serde(alias = "m_Name")]
    name: String,

    categories: Vec<AssetCategory>,
    #[serde(alias = "rawAssets")]
    raw_assets: HashSet<Asset>,
}

impl Manifest {
    fn new() -> Manifest {
        Manifest { 
            game_object: UnityHeader { 
                path_id: 0, 
                file_id: 0 
            }, 
            enabled: 1, 
            script: UnityHeader { 
                path_id: 3126599977334250442, 
                file_id: 0 
            }, 
            name: String::from("manifest"), 
            categories: vec![
                AssetCategory::new("Master"),
                AssetCategory::new("Others")
            ], 
            raw_assets: HashSet::new() 
        }
    }
}

fn deserialize_file(path: &Path) -> Result<Manifest, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let manifest: Manifest = serde_json::from_reader(reader)?;

    Ok(manifest)
}

fn build_manifest(manifest_path: &Path, manifest_type: &str) -> Manifest {
    let mut result = Manifest::new();
    let total_folders = manifest_path.read_dir().unwrap().count();

    let folders = manifest_path.read_dir().unwrap();

    for (i, entry) in folders.enumerate() {

        println!("{} / {}", i, total_folders);

        let json_path = entry.unwrap().path().join(manifest_type);
        let manifest = match deserialize_file(&json_path) {
            Err(why) => panic!("Failed to deserialize JSON file {}: {}", json_path.display(), why),
            Ok(m) => m
        };

        for category in manifest.categories {
            let mut result_category = result.categories
                .iter_mut()
                .find(|c| { 
                    c.name == category.name 
                }).unwrap();

            for asset in category.assets {
                result_category.assets.get_or_insert(asset);
            }

            // encrypted_assets is unused
        }

        for asset in manifest.raw_assets {
            result.raw_assets.get_or_insert(asset);
        }
    };

    result
}

fn main() {
    println!("Starting");
    let manifest_path = Path::new(MANIFEST_PATH);

    let all_manifest_types = vec![
        "assetbundle.manifest.json", 
        "assetbundle.en_us.manifest.json", 
        "assetbundle.en_eu.manifest.json", 
        "assetbundle.zh_tw.manifest.json", 
        "assetbundle.zh_cn.manifest.json"
    ];

    for manifest_type in all_manifest_types {
        println!("Processing manifests of type {}", manifest_type);
        let manifest = build_manifest(manifest_path, manifest_type);
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        match fs::write(Path::new(manifest_type), &json) {
            Err(why) => panic!("Failed to write result to path {}: {}", manifest_type, why),
            Ok(_) => ()
        };
    }
}

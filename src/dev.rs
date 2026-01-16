use std::{collections::HashMap, fmt::Display, process::Command};

use regex::Regex;

use crate::config::XFixTouchscreen;

#[allow(unused)]
// const TOUCHSCREEN_TYPE: &str = "ID_INPUT_TOUCHSCREEN";
const TOUCHSCREEN_TYPE: &str = "ID_INPUT_MOUSE";

pub struct XFixEventNode {
    path: String,
    properties: HashMap<String, String>,
}

impl XFixEventNode {
    pub fn new(path: String, model_name: String, vendor: String, id_path: String) -> Self {
        Self {
            path,
            properties: [
                ("ID_MODEL".to_string(), model_name),
                ("ID_VENDOR".to_string(), vendor),
                ("ID_PATH".to_string(), id_path),
            ]
            .into(),
        }
    }

    pub fn event_path(&self) -> &str {
        &self.path
    }

    pub fn vendor(&self) -> Option<&str> {
        self.properties.get("ID_VENDOR").map(|s| s.as_str())
    }

    pub fn model(&self) -> Option<&str> {
        self.properties.get("ID_MODEL").map(|s| s.as_str())
    }

    pub fn id_path(&self) -> Option<&str> {
        self.properties.get("ID_PATH").map(|s| s.as_str())
    }

    pub fn to_touchscreen(
        &self,
        mapping: Option<String>,
    ) -> Result<XFixTouchscreen, Box<dyn std::error::Error>> {
        Ok(XFixTouchscreen {
            vendor: self.vendor().ok_or("Vendor not found")?.to_string(),
            id_path: self.id_path().ok_or("ID_PATH not found")?.to_string(),
            map_to_output: mapping,
        })
    }
}

impl Display for XFixEventNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(vendor) = self.vendor() {
            write!(f, "{} - ", vendor)?;
        }

        if let Some(model) = self.model() {
            write!(f, "{} ({})", model, self.path)
        } else {
            write!(f, "{}", self.path)
        }
    }
}

#[derive(Debug)]
pub struct XFixTouchscreenWithNode<'a> {
    screen: &'a XFixTouchscreen,
    node: Option<String>,
}

#[derive(Debug)]
pub struct XFixTouchscreenWithXinputId<'a> {
    screen: XFixTouchscreenWithNode<'a>,
    id: Option<u32>,
}

pub fn find_all_touchscreens_nodes() -> Result<Vec<XFixEventNode>, Box<dyn std::error::Error>> {
    #[cfg(not(target_os = "linux"))]
    {
        Ok(vec![
            XFixEventNode::new(
                "/dev/input/event0".to_string(),
                "Touchscreen 1".to_string(),
                "Manufacturer".to_string(),
                "ID_PATH1".to_string(),
            ),
            XFixEventNode::new(
                "/dev/input/event1".to_string(),
                "Touchscreen 2".to_string(),
                "Manufacturer".to_string(),
                "ID_PATH2".to_string(),
            ),
        ])
    }

    #[cfg(target_os = "linux")]
    {
        let all_touchscreen_nodes = glob::glob("/dev/input/event*")?
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|path| {
                let device_name = path.display().to_string();
                let udev_output = Command::new("udevadm")
                    .args(["info", "--query=property"])
                    .arg(format!("--name={}", device_name))
                    .output()
                    .ok()?;

                let udev_output_str = String::from_utf8(udev_output.stdout).ok()?;

                let udev_properties = udev_output_str
                    .lines()
                    .into_iter()
                    .filter_map(|line| line.split_once("="))
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect::<HashMap<_, _>>();

                Some((device_name, udev_properties))
            })
            .filter(|(_, props)| props.get(TOUCHSCREEN_TYPE).is_some_and(|val| val == "1"))
            .map(|(path, properties)| XFixEventNode { path, properties })
            .collect::<Vec<_>>();

        Ok(all_touchscreen_nodes)
    }
}

pub fn find_touchscreen_nodes<'a>(
    screens: impl IntoIterator<Item = &'a XFixTouchscreen>,
) -> Result<Vec<XFixTouchscreenWithNode<'a>>, Box<dyn std::error::Error>> {
    let all_touchscreen_nodes = find_all_touchscreens_nodes()?;

    let screens = screens
        .into_iter()
        .map(|s| {
            let node = all_touchscreen_nodes
                .iter()
                .find(|node| {
                    node.vendor().is_some_and(|vendor| vendor == &s.vendor)
                        && node.id_path().is_some_and(|path| path == &s.id_path)
                })
                .map(|node| node.path.clone());

            XFixTouchscreenWithNode { screen: s, node }
        })
        .collect::<Vec<_>>();

    Ok(screens)
}

pub fn find_xinput_id<'a>(
    screens: Vec<XFixTouchscreenWithNode<'a>>,
) -> Result<Vec<XFixTouchscreenWithXinputId<'a>>, Box<dyn std::error::Error>> {
    let xinput_output = Command::new("xinput").args(["list", "--short"]).output()?;
    let xinput_output_str = str::from_utf8(&xinput_output.stdout)?;

    let pointer_devices_regex =
        Regex::new("Virtual core pointer(.*)\n(?<devices>(.*\n)*)(.*)Virtual core keyboard")?;
    let devices_str = &pointer_devices_regex.captures(xinput_output_str).unwrap()["devices"];

    let device_id_regex = Regex::new(".*id=(?<id>[0-9]+).*")?;
    let device_node_regex = Regex::new("Device Node.*\"(?<node>[^\"]+)\"")?;

    let device_mapping = devices_str
        .lines()
        .into_iter()
        .filter_map(|line| {
            let device_id = device_id_regex.captures(line)?["id"].parse::<u32>().ok()?;
            Some(device_id)
        })
        .filter_map(|device_id| {
            let xinput_props_output = Command::new("xinput")
                .arg("--list-props")
                .arg(device_id.to_string())
                .output()
                .ok()?;
            let xinput_props_output_str = str::from_utf8(&xinput_props_output.stdout).ok()?;
            let device_node =
                device_node_regex.captures(xinput_props_output_str)?["node"].to_string();

            Some((device_node, device_id))
        })
        .collect::<HashMap<_, _>>();

    let screens = screens
        .into_iter()
        .map(|screen| {
            let id = screen
                .node
                .as_ref()
                .and_then(|node| device_mapping.get(node).copied());

            XFixTouchscreenWithXinputId { screen, id }
        })
        .collect::<Vec<_>>();

    Ok(screens)
}

pub fn assign_screens_to_outputs(screens: Vec<XFixTouchscreenWithXinputId<'_>>) {
    for screen in screens {
        let (Some(xinput_id), Some(output)) =
            (screen.id, screen.screen.screen.map_to_output.as_ref())
        else {
            continue;
        };

        println!(
            "[xfix] Mapping device with xinput id {} to output {:?}",
            xinput_id, output
        );

        Command::new("xinput")
            .arg("map-to-output")
            .arg(xinput_id.to_string())
            .arg(output);
    }
}

pub fn find_connected_video_outputs() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    #[cfg(not(target_os = "linux"))]
    {
        Ok(vec![
            "DP-1".to_string(),
            "HDMI-1".to_string(),
            "VGA-1".to_string(),
        ])
    }

    #[cfg(target_os = "linux")]
    {
        let output_list = Command::new("xrandr").output()?;
        let output_list_str = str::from_utf8(&output_list.stdout)?;

        let output_regex = Regex::new(r"^(?<output>.*).*(\sconnected).*$").unwrap();

        let connected_outputs = output_list_str
            .lines()
            .filter_map(|line| {
                let captures = output_regex.captures(line)?;
                Some(captures["output"].to_string())
            })
            .collect::<Vec<_>>();

        Ok(connected_outputs)
    }
}

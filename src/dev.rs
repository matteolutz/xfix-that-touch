use std::{collections::HashMap, process::Command};

use regex::Regex;

use crate::config::XFixTouchscreen;

const TOUCHSCREEN_TYPE: &str = "ID_INPUT_TOUCHSCREEN";

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

pub fn find_touchscreen_nodes<'a>(
    screens: &'a [XFixTouchscreen],
) -> Result<Vec<XFixTouchscreenWithNode<'a>>, Box<dyn std::error::Error>> {
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
        .collect::<Vec<_>>();

    let screens = screens
        .iter()
        .map(|s| {
            let node = all_touchscreen_nodes
                .iter()
                .find(|(_, props)| {
                    props
                        .get("ID_VENDOR")
                        .is_some_and(|vendor| vendor == &s.vendor)
                        && props.get("ID_PATH").is_some_and(|path| path == &s.id_path)
                })
                .map(|(node, _)| node.clone());

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

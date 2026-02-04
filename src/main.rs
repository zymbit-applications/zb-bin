// -------------------------------------------------------------------------------------------------------
// Copyright (C) 2023, 2024 Zymbit. All rights reserved.
// Use of this software and associated documentation files (the "Software") is subject to Zymbit
// terms and conditions, and license, found here:
//
// https://www.zymbit.com/terms-and-conditions-of-sale-general/
// https://www.zymbit.com/software-license-general/
//
// Permission to install, use, copy, and modify this software and its documentation for educational,
// research, and not-for-profit purposes, without fee and without a signed licensing agreement,
// is hereby granted, provided that the above copyright notice, this paragraph and the following
// three paragraphs appear in all copies, modifications, and distributions.  Commercial use
// of any kind requires a written license from Zymbit. Redistribution of this software in original or
// modified form requires a written license from Zymbit. Refer to full license for details.
// IN NO EVENT SHALL ZYMBIT INC. OR ITS AGENTS BE LIABLE TO ANY PARTY FOR DIRECT, INDIRECT,
// SPECIAL, INCIDENTAL, OR CONSEQUENTIAL DAMAGES, INCLUDING LOST PROFITS, ARISING OUT OF
// THE USE OF THIS SOFTWARE AND ITS DOCUMENTATION, EVEN IF ZYMBIT HAS BEEN ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
// ZYMBIT SPECIFICALLY DISCLAIMS ANY WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE. THE SOFTWARE AND
// ACCOMPANYING DOCUMENTATION, IF ANY, PROVIDED HEREUNDER IS PROVIDED "AS IS". ZYMBIT HAS
// NO OBLIGATION TO PROVIDE MAINTENANCE, SUPPORT, UPDATES, ENHANCEMENTS, OR MODIFICATIONS.
//
// You may not use any Zymbit products in life-critical equipment unless authorized officers
// of the parties have executed a special contract specifically governing such use.
// -------------------------------------------------------------------------------------------------------

#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::enum_variant_names)]

use std::process;

use crate::system::PiModule;
use anyhow::{Context, Result};
use dialoguer::theme::ColorfulTheme;
use terminal::{formatted_left_output, OutputColor};

mod installer_cli;
mod system;
mod terminal;
mod toolchain;
mod zbcli;

async fn start() -> Result<()> {
    let cli_args = installer_cli::parse_args()?;
    let system = system::System::get(cli_args.rpi_model)?;
    println!("{system}");

    let should_use_hardware = match cli_args.use_hw {
        Some(flag) => flag,
        None => dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "'zbcli' comes with software signing by default. Include hardware key signing? \
                    (Requires SCM or HSM6)",
            )
            .item("Yes")
            .item("No")
            .default(1)
            .interact()
            .context("Failed to get signing option")?
            == 0,
    };

    let target_asset = match system.pi_module {
        PiModule::Rpi0_64 => {
            if should_use_hardware {
                zbcli::ZbcliAsset::Rpi0Hardware
            } else {
                zbcli::ZbcliAsset::Rpi0
            }
        }
        PiModule::Rpi4_64 => {
            if should_use_hardware {
                zbcli::ZbcliAsset::Rpi4Hardware
            } else {
                zbcli::ZbcliAsset::Rpi4
            }
        }
        PiModule::Rpi5_64 => {
            if should_use_hardware {
                zbcli::ZbcliAsset::Rpi5Hardware
            } else {
                zbcli::ZbcliAsset::Rpi5
            }
        }
    };

    toolchain::install::prompt("zbcli", &target_asset.to_string(), cli_args.zb_version.as_ref()).await?;

    println!(
        "Installed zbcli. Run 'zbcli install' to install Bootware onto \
    your system or 'zbcli --help' for more options."
    );

    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = start().await.map_err(|e| {
        eprintln!(
            "{} {:?}",
            formatted_left_output("Error", &OutputColor::Red),
            e
        );

        process::exit(1);
    });
}

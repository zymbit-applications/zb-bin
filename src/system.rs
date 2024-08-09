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

use anyhow::{bail, Context, Result};
use derive_more::Display;
use std::fmt::Display;
use std::{
    fs::{self},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub struct System {
    pub os: OperatingSystem,
    pub pi_module: PiModule,
    pub zymbit_module: ZymbitModule,
}

impl System {
    pub fn get() -> Result<System> {
        Ok(System {
            os: OperatingSystem::get()?,
            pi_module: PiModule::get()?,
            zymbit_module: ZymbitModule::get()?,
        })
    }

    #[must_use]
    pub fn kernel(&self) -> Kernel {
        if self.os == OperatingSystem::Ubuntu {
            return Kernel::Vmlinuz;
        }

        match self.pi_module {
            PiModule::Rpi4_64 => Kernel::Kernel8img,
            PiModule::Rpi5_64 => Kernel::Kernel2712img,
        }
    }
}

impl Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\t---------\n\
            \tPi Module         {}\n\
            \tOperating System  {}\n\
            \tZymbit module     {}\n\
            \tKernel            {}\n\
            \t---------\n",
            self.pi_module,
            self.os,
            self.zymbit_module,
            self.kernel()
        )
    }
}

fn is_ubuntu() -> bool {
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        contents.contains("Ubuntu")
    } else {
        false
    }
}

fn is_kernel8_img() -> bool {
    let boot_path = "/boot";
    if let Ok(entries) = fs::read_dir(boot_path) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name == "kernel8.img" {
                    return true;
                }
            }
        }
    }

    false
}

#[derive(Display, PartialEq)]
pub enum OperatingSystem {
    #[display(fmt = "Ubuntu")]
    Ubuntu,

    #[display(fmt = "Rpi-Bullseye")]
    RpiBullseye,

    #[display(fmt = "Rpi-Bookworm")]
    RpiBookworm,
}

impl OperatingSystem {
    fn get() -> Result<OperatingSystem> {
        let os_release = fs_extra::file::read_to_string(Path::new("/etc").join("os-release"))?;

        if is_ubuntu() {
            Ok(OperatingSystem::Ubuntu)
        } else if os_release.contains("bookworm") {
            Ok(OperatingSystem::RpiBookworm)
        } else if is_kernel8_img() {
            Ok(OperatingSystem::RpiBullseye)
        } else {
            bail!(
                "Unsupported distribution. Double check you are running in Ubuntu or Rasberry Pi."
            )
        }
    }
}

#[derive(Display)]
pub enum Kernel {
    #[display(fmt = "vmlinuz")]
    Vmlinuz,

    #[display(fmt = "kernel8.img")]
    Kernel8img,

    #[display(fmt = "kernel_2712.img")]
    Kernel2712img,
}

/// Equivalent to `chmod +x`
pub fn add_executable_permission(file: &PathBuf) -> Result<()> {
    let metadata =
        fs::metadata(file).context(format!("Failed to get metadata ({})", file.display()))?;
    let mut permissions = metadata.permissions();

    let mode = permissions.mode();
    permissions.set_mode(mode | 0o111);

    fs::set_permissions(file, permissions).context(format!(
        "Failed to set executable permissions ({})",
        file.display()
    ))?;

    Ok(())
}

/// Module for both Pi and Compute Module
#[derive(Display, PartialEq)]
pub enum PiModule {
    /// Pi 4 or CM 4
    #[display(fmt = "Rasberry Pi 4")]
    Rpi4_64,

    /// Pi 5 or CM 5
    #[display(fmt = "Rasberry Pi 5")]
    Rpi5_64,
}

impl PiModule {
    fn get() -> Result<PiModule> {
        #[cfg(feature = "rpi4")]
        {
            let model = fs_extra::file::read_to_string(
                Path::new("/proc").join("device-tree").join("model"),
            )?;

            if model.contains("Rasbperry Pi 5") {
                bail!("Raspberry Pi 4 version of zb installed, but Rasberry Pi 4 is not detected. Ensure you have the correct zb version installed");
            }

            // Need explicit return when rpi4 and rpi5 are both set for installer
            #[allow(clippy::needless_return)]
            return Ok(PiModule::Rpi4_64);
        }

        #[allow(unreachable_code)]
        #[cfg(feature = "rpi5")]
        {
            let model = fs_extra::file::read_to_string(
                Path::new("/proc").join("device-tree").join("model"),
            )?;

            if !model.contains("Rasbperry Pi 5") {
                bail!("Raspberry Pi 5 version of zb installed, but Rasberry Pi 5 is not detected. Ensure you have the correct zb version installed");
            }

            Ok(PiModule::Rpi5_64)
        }
    }
}

#[derive(Display, PartialEq)]
pub enum ZymbitModule {
    #[display(fmt = "Zymkey")]
    Zymkey,

    #[display(fmt = "Secure Compute Module")]
    Scm,
}

impl ZymbitModule {
    fn get() -> Result<ZymbitModule> {
        if glob::glob("/dev/zscm*")
            .context("Failed to check '/dev/zscm*'")?
            .count()
            > 0
        {
            Ok(ZymbitModule::Scm)
        } else {
            Ok(ZymbitModule::Zymkey)
        }
    }
}

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
    path::{
        Path,
        PathBuf,
    },
};

static ROOTDEV_NAME: &str = "mmcblk0";

pub struct System {
    pub os: OperatingSystem,
    pub pi_module: PiModule,
    pub zymbit_module: ZymbitModule,
    // pub disk_layout: DiskLayout,
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

#[derive(Display, PartialEq, Debug)]
pub enum PiModule {
    /// Pi Zero 2 W
    #[display(fmt = "Raspberry Pi Zero 2 W")]
    Rpi0_64,

    /// Pi 4 or CM 4
    #[display(fmt = "Raspberry Pi 4/Compute Module 4")]
    Rpi4_64,

    /// Pi 5 or CM 5
    #[display(fmt = "Raspberry Pi 5/Compute Module 5")]
    Rpi5_64,
}

#[derive(Display, PartialEq)]
pub enum ZymbitModule {
    #[display(fmt = "Zymkey")]
    Zymkey,

    #[display(fmt = "Secure Compute Module")]
    Scm,
    // #[display(fmt = "Hardware Security Module 6")]
    // HSM6,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct DiskLayout {
    pub a_within_half: bool,
    pub cryptroot: bool,
    pub one_root_fs: bool,
    pub active_root: PathBuf,
    pub boot_mountpoint: PathBuf,
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

impl System {
    pub fn get(pi_mod_override: Option<PiModule>) -> Result<Self> {
        Ok(Self {
            os: OperatingSystem::get()?,
            // use override passed on CLI if present, else try to autodetect, else return autodetection error
            pi_module: pi_mod_override
                .map(|pm| Ok(pm))
                .unwrap_or_else(|| PiModule::get())?,
            zymbit_module: ZymbitModule::get()?,
            // disk_layout: DiskLayout::get()?,
        })
    }

    #[must_use]
    pub fn kernel(&self) -> Kernel {
        if self.os == OperatingSystem::Ubuntu {
            return Kernel::Vmlinuz;
        }

        match &self.pi_module {
            PiModule::Rpi0_64 | PiModule::Rpi4_64 => Kernel::Kernel8img,
            PiModule::Rpi5_64 => Kernel::Kernel2712img,
        }
    }
}

impl Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\t---------\n\
            \tPi Module:         {}\n\
            \tOperating System:  {}\n\
            \tZymbit module:     {}\n\
            \tKernel:            {}\n\
            \t---------\n",
            self.pi_module,
            self.os,
            self.zymbit_module,
            self.kernel()
        )
    }
}

impl OperatingSystem {
    fn get() -> Result<Self> {
        use OperatingSystem::{RpiBookworm, RpiBullseye, Ubuntu};
        let os_rel = os_release()?;
        if os_rel.contains("Ubuntu") {
            Ok(Ubuntu)
        } else if os_rel.contains("bookworm") {
            Ok(RpiBookworm)
        } else if os_rel.contains("bullseye") {
            Ok(RpiBullseye)
        } else {
            bail!("Unsupported OS platform. Only the official RPi Debian and Ubuntu Linux releases are supported.")
        }
    }
}

impl PiModule {
    fn get() -> Result<Self> {
        use PiModule::{Rpi0_64, Rpi4_64, Rpi5_64};

        let model = fs::read_to_string("/sys/firmware/devicetree/base/model")
            .context("unable to retrieve host platform information from devicetree. (Hint: set the `--rpi-model` flag)")?;

        Ok(
            if model.contains("Raspberry Pi 5") || model.contains("Compute Module 5") {
                Rpi5_64
            } else if model.contains("Raspberry Pi 4") || model.contains("Compute Module 4") {
                Rpi4_64
            } else if model.contains("Pi Zero 2 W") {
                Rpi0_64
            } else {
                bail!(
                    "Unknown host platform in devicetree: '{}'. (Hint: set the `--rpi-model` flag)",
                    model
                );
            },
        )
    }
}

impl ZymbitModule {
    // TODO: figure out a better way to get the module (use C API probably)
    //  and properly detect an HSM6
    fn get() -> Result<Self> {
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

#[allow(dead_code)]
impl DiskLayout {
    pub fn get() -> Result<Self> {
        // TODO -- example values used here
        Ok(Self {
            a_within_half: true,
            cryptroot: true,
            one_root_fs: true,
            active_root: PathBuf::from("/dev/mmcblk0p2"),
            boot_mountpoint: boot_mountpoint()?,
        })
    }
}

/// Equivalent to `chmod a+x`
pub fn add_executable_permission(file: &Path) -> Result<()> {
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

#[allow(dead_code)]
fn boot_mountpoint() -> Result<PathBuf> {
    let Ok(mounts) = fs::read_to_string("/etc/mtab") else {
        bail!("unable to read system mount table");
    };
    let mountpoint = mounts
        .split('\n')
        .filter(|&entry| entry.contains(ROOTDEV_NAME))
        .map(|entry| {
            entry
                .split_whitespace()
                .nth(1)
                .unwrap_or_else(|| panic!("unable to parse system mount table"))
        })
        .next()
        .unwrap_or_else(|| panic!("{} not mounted!", ROOTDEV_NAME));

    Ok(PathBuf::from(mountpoint))
}

fn os_release() -> Result<String> {
    fs::read_to_string("/etc/os-release").context("unable to determine OS type")
}

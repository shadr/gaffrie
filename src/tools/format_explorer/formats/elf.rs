use nom::{
    bytes::complete::take,
    number::complete::{be_u16, be_u32, be_u64, le_u16, le_u32, le_u64, u8},
    sequence::Tuple,
    IResult,
};

use crate::tools::format_explorer::FileFormatUi;

pub struct ElfFormat {
    pub mag: [u8; 4],
    pub class: u8,
    pub data: u8,
    pub ei_version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub pad: [u8; 7],
    pub type_: u16,
    pub machine: u16,
    pub e_version: u32,
    pub entry: u64,
    pub ph_offset: u64,
    pub sh_offset: u64,
    pub flags: u32,
    pub eh_size: u16,
    pub ph_entry_size: u16,
    pub ph_entry_num: u16,
    pub sh_entry_size: u16,
    pub sh_entry_num: u16,
    pub sh_str_offset: u16,
}

impl FileFormatUi for ElfFormat {
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) {
        ui.collapsing(name, |ui| {
            // mag
            self.class.ui(ui, "class");
            self.data.ui(ui, "data");
            self.ei_version.ui(ui, "ei_version");
            self.os_abi.ui(ui, "os_abi");
            self.abi_version.ui(ui, "abi_version");
            // pad
            self.type_.ui(ui, "type");
            self.machine.ui(ui, "machine");
            self.e_version.ui(ui, "e_version");
            self.entry.ui(ui, "entry");
            self.ph_offset.ui(ui, "ph_offset");
            self.sh_offset.ui(ui, "sh_offset");
            self.flags.ui(ui, "flags");
            self.eh_size.ui(ui, "eh_size");
            self.ph_entry_size.ui(ui, "ph_entry_size");
            self.ph_entry_num.ui(ui, "ph_entry_num");
            self.sh_entry_size.ui(ui, "sh_entry_size");
            self.sh_entry_num.ui(ui, "sh_entry_num");
            self.sh_str_offset.ui(ui, "sh_str_offset");
        });
    }
}

impl ElfFormat {
    pub fn parse(tail: &[u8]) -> IResult<&[u8], Self> {
        let (tail, (mag, class, data, ei_version, os_abi, abi_version, pad)) =
            (take(4usize), u8, u8, u8, u8, u8, take(7usize)).parse(tail)?;
        let (fu16, fu32, fu64) = if data == 1 {
            (
                le_u16 as fn(_) -> _,
                le_u32 as fn(_) -> IResult<_, u32>,
                le_u64 as fn(_) -> IResult<_, u64>,
            )
        } else if data == 2 {
            (
                be_u16 as fn(_) -> _,
                be_u32 as fn(_) -> IResult<_, u32>,
                be_u64 as fn(_) -> IResult<_, u64>,
            )
        } else {
            unimplemented!()
        };
        let (tail, (type_, machine, e_version)) = (fu16, fu16, fu32).parse(tail)?;
        let (tail, (entry, ph_offset, sh_offset)) = if class == 1 {
            let (tail, (entry, ph, sh)) = (fu32, fu32, fu32).parse(tail)?;
            (tail, (entry as u64, ph as u64, sh as u64))
        } else if class == 2 {
            (fu64, fu64, fu64).parse(tail)?
        } else {
            unimplemented!()
        };
        let (
            tail,
            (
                flags,
                eh_size,
                ph_entry_size,
                ph_entry_num,
                sh_entry_size,
                sh_entry_num,
                sh_str_offset,
            ),
        ) = (fu32, fu16, fu16, fu16, fu16, fu16, fu16).parse(tail)?;
        Ok((
            tail,
            Self {
                mag: mag.try_into().unwrap(),
                class,
                data,
                ei_version,
                os_abi,
                abi_version,
                pad: pad.try_into().unwrap(),
                type_,
                machine,
                e_version,
                entry,
                ph_offset,
                sh_offset,
                flags,
                eh_size,
                ph_entry_size,
                ph_entry_num,
                sh_entry_size,
                sh_entry_num,
                sh_str_offset,
            },
        ))
    }

    pub fn new(bytes: &[u8]) -> Self {
        Self::parse(bytes).unwrap().1
    }
}

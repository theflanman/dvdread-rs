use std::os::raw::{c_char, c_int, c_uchar, c_uint};
use std::path::Path;
use crate::{dvd_file_t, dvd_read_domain_t, dvd_read_domain_t_DVD_READ_INFO_BACKUP_FILE, dvd_read_domain_t_DVD_READ_INFO_FILE, dvd_read_domain_t_DVD_READ_MENU_VOBS, dvd_read_domain_t_DVD_READ_TITLE_VOBS, dvd_reader_t, dvd_stat_t, ifoOpen, ifoOpenVMGI, ifoOpenVTSI, ifo_handle_t, ifo_print, DVDClose, DVDDiscID, DVDFileStat, DVDISOVolumeInfo, DVDOpen, DVDOpenFile, DVDUDFCacheLevel, DVDUDFVolumeInfo, UDFFindFile, UDFGetVolumeIdentifier, UDFGetVolumeSetIdentifier};

pub enum DvdDomain {
    InfoFile = dvd_read_domain_t_DVD_READ_INFO_FILE as isize,
    BackupFile = dvd_read_domain_t_DVD_READ_INFO_BACKUP_FILE as isize,
    MenuVobs = dvd_read_domain_t_DVD_READ_MENU_VOBS as isize,
    TitleVobs = dvd_read_domain_t_DVD_READ_TITLE_VOBS as isize,
}

#[derive(Debug)]
pub struct DvdReader {
    reader: dvd_reader_t,
}

impl DvdReader {

    /// Create a new `DvdReader` from a path pointing to a block device, directory, or ISO/UDF/etc archive.
    ///
    /// Opens a block device of a DVD-ROM file, or an image file, or a directory
    /// name for a mounted DVD or HD copy of a DVD.
    /// The second form of Open function (DVDOpenStream) can be used to
    /// provide custom stream_cb functions to access the DVD (see libdvdcss).
    ///
    /// If the given file is a block device, or is the mountpoint for a block
    /// device, then that device is used for CSS authentication using libdvdcss.
    /// If no device is available, then no CSS authentication is performed,
    /// and we hope that the image is decrypted.
    ///
    /// If the path given is a directory, then the files in that directory may be
    /// in any one of these formats:
    ///
    /// * path/VIDEO_TS/VTS_01_1.VOB
    /// * path/video_ts/vts_01_1.vob
    /// * path/VTS_01_1.VOB
    /// * path/vts_01_1.vob
    pub fn new(file_name: Box<Path>) -> DvdReader {
        DvdReader { reader: unsafe { *DVDOpen(file_name.to_str().unwrap().as_ptr() as *const c_char) } }
    }

    /// Close this reader
    /// Closes and cleans up the DVD reader object.
    ///
    /// You must close all open files before calling this function.
    ///
    /// @param dvd A read handle that should be closed.
    ///
    /// DVDClose(dvd);
    pub fn close(&mut self) {
        unsafe {DVDClose(&mut self.reader)}
    }

    /// Get information about the file for a title/domain combination.
    ///
    /// Stats a file on the DVD given the title number and domain.
    /// The information about the file is stored in a dvd_stat_t
    /// which contains information about the size of the file and
    /// the number of parts in case of a multipart file and the respective
    /// sizes of the parts.
    /// A multipart file is for instance VTS_02_1.VOB, VTS_02_2.VOB, VTS_02_3.VOB
    /// The size of VTS_02_1.VOB will be stored in stat->parts_size[0],
    /// VTS_02_2.VOB in stat->parts_size[1], ...
    /// The total size (sum of all parts) is stored in stat->size and
    /// stat->nr_parts will hold the number of parts.
    /// Only DVD_READ_TITLE_VOBS (VTS_??_[1-9].VOB) can be multipart files.
    ///
    /// This function is only of use if you want to get the size of each file
    /// in the filesystem. These sizes are not needed to use any other
    /// functions in libdvdread.
    ///
    /// @param dvd  A dvd read handle.
    /// @param titlenum Which Video Title Set should be used, VIDEO_TS is 0.
    /// @param domain Which domain.
    /// @param stat Pointer to where the result is stored.
    /// @return If successful 0, otherwise -1.
    ///
    /// ```
    /// use std::path::Path;
    /// use dvdread_rs::dvd_reader::DvdDomain;
    /// use dvdread_rs::dvd_reader::DvdReader;
    /// use dvdread_rs::dvd_stat_t;
    ///
    ///
    /// let mut dvd = DvdReader::new(Box::from(Path::new("./image.iso")));
    ///
    /// let file_stat = match dvd.file_stat(1, DvdDomain::InfoFile) {
    ///     Ok(stat) => {stat},
    ///     Err(e) => {panic!("{}", e)},
    /// };
    /// ```
    pub fn file_stat(&mut self, title_num: usize, domain: DvdDomain) -> Result<dvd_stat_t, String>{
        let mut stat: dvd_stat_t = dvd_stat_t {
            size: 0,
            nr_parts: 0,
            parts_size: [0; 9],
        };
        let result =  unsafe {
            DVDFileStat(
                &mut self.reader,
                title_num as c_int,
                domain as dvd_read_domain_t,
                &mut stat
            )
        };

        if result == 0 {
            Ok(stat)
        } else {
            Err(format!("Error opening file: {}", result))
        }

    }

    /// Opens a file on the DVD given the title number and domain.
    ///
    /// If the title number is 0, the video manager information is opened
    /// (VIDEO_TS.[IFO,BUP,VOB]).  Returns a file structure which may be
    /// used for reads, or 0 if the file was not found.
    ///
    /// @param dvd  A dvd read handle.
    /// @param titlenum Which Video Title Set should be used, VIDEO_TS is 0.
    /// @param domain Which domain.
    /// @return If successful a a file read handle is returned, otherwise 0.
    ///
    /// dvd_file = DVDOpenFile(dvd, titlenum, domain); */
    pub fn open_file(&mut self, title_number: usize, domain: DvdDomain) -> Result<dvd_file_t, String> {
        Ok(unsafe {
            *DVDOpenFile(
                &mut self.reader,
                title_number as c_int,
                domain as dvd_read_domain_t,
            )
        })
    }

    /// Get the ID for this disc volume.
    /// Get a unique 128 bit disc ID.
    /// This is the MD5 sum of VIDEO_TS.IFO and the VTS_0?_0.IFO files
    /// in title order (those that exist).
    /// If you need a 'text' representation of the id, print it as a
    /// hexadecimal number, using lowercase letters, discid[0] first.
    /// I.e. the same format as the command-line 'md5sum' program uses.
    ///
    /// @param dvd A read handle to get the disc ID from
    /// @param discid The buffer to put the disc ID into. The buffer must
    ///               have room for 128 bits (16 chars).
    /// @return 0 on success, -1 on error.
    pub fn disc_id(&mut self) -> Result<String, String> {
        let mut c_string_ptr: c_uchar = 0;
        let result = unsafe { DVDDiscID(
            &mut self.reader,
            &mut c_string_ptr,
        ) };

        if result == 0 {
            Ok(c_string_ptr.to_string())
        } else {
            Err(format!("Error opening file: {}", result))
        }
    }

    /// Get the UDF VolumeIdentifier and VolumeSetIdentifier
    /// from the PrimaryVolumeDescriptor.
    ///
    /// # Arguments
    /// * `volid_size` No more than volid_size bytes will be copied to volid.
    ///                   If the VolumeIdentifier is truncated because of this
    ///                   it will still be null terminated.
    /// * `volsetid_size` At most volsetid_size bytes will be copied to volsetid.
    ///
    /// # Returns
    /// * `volid` The buffer to put the VolumeIdentifier into.
    ///              The VolumeIdentifier is latin-1 encoded (8bit unicode)
    ///              null terminated and max 32 bytes (including '\0')
    /// * `volsetid` The buffer to put the VolumeSetIdentifier into.
    ///                 The VolumeIdentifier is 128 bytes as
    ///                 stored in the UDF PrimaryVolumeDescriptor.
    ///                 Note that this is not a null terminated string.
    pub fn udf_volume_info(&mut self, volid_size: usize, volsetid_size: usize) -> Result<(String, u128), String> {
        let mut volid = [0u8; 32];
        let mut volsetid = [0u8; 16];

        let result = unsafe {
            DVDUDFVolumeInfo(
                &mut self.reader,
                volid.as_mut_ptr() as *mut c_char,
                volid_size as c_uint,
                volsetid.as_mut_ptr(),
                volsetid_size as c_uint,
            )
        };

        if result == 0 {
            Ok((
                unsafe { String::from_utf8(volid.to_vec()) }.unwrap(),
                {
                    let mut volsetidout = 0u128;
                    for i in 0..16 {
                        volsetidout = volsetidout + (volsetid[i] as u128);
                        if i < 15 {
                            volsetidout = volsetidout << 8;
                        }
                    }
                    volsetidout
                },
            ))
        } else {
            Err(format!("Error opening file: {}", result))
        }

    }

    /// Get the ISO9660 VolumeIdentifier and VolumeSetIdentifier
    ///
    /// * Only use this function as fallback if DVDUDFVolumeInfo returns -1
    /// * this will happen on a disc mastered only with an iso9660 filesystem
    /// * All video DVD discs have UDF filesystem
    ///
    /// @param dvd A read handle to get the disc ID from
    /// @param volid The buffer to put the VolumeIdentifier into.
    ///              The VolumeIdentifier is coded with '0-9','A-Z','_'
    ///              null terminated and max 33 bytes (including '\0')
    /// @param volid_size No more than volid_size bytes will be copied to volid.
    ///                   If the VolumeIdentifier is truncated because of this
    ///                   it will still be null terminated.
    /// @param volsetid The buffer to put the VolumeSetIdentifier into.
    ///                 The VolumeIdentifier is 128 bytes as
    ///                 stored in the ISO9660 PrimaryVolumeDescriptor.
    ///                 Note that this is not a null terminated string.
    /// @param volsetid_size At most volsetid_size bytes will be copied to volsetid.
    /// @return 0 on success, -1 on error.
    pub fn iso_volume_info(&mut self, volid_size: usize, volsetid_size: usize) -> Result<(String, u128), String> {
        let mut volid = [0u8; 32];
        let mut volsetid = [0u8; 16];

        let result = unsafe {
            DVDISOVolumeInfo(
                &mut self.reader,
                volid.as_mut_ptr() as *mut c_char,
                volid_size as c_uint,
                volsetid.as_mut_ptr(),
                volsetid_size as c_uint,
            )
        };

        if result == 0 {
            Ok((
                unsafe { String::from_utf8(volid.to_vec()).unwrap() },
                {
                    let mut volsetidout = 0u128;
                    for i in 0..16 {
                        volsetidout = volsetidout + (volsetid[i] as u128);
                        if i < 15 {
                            volsetidout = volsetidout << 8;
                        }
                    }
                    volsetidout
                },
            ))
        } else {
            Err(format!("Error opening file: {}", result))
        }
    }

    /// Sets the level of caching that is done when reading from a device
    ///
    /// @param dvd A read handle to get the disc ID from
    /// @param level The level of caching wanted.
    ///             -1 - returns the current setting.
    ///              0 - UDF Cache turned off.
    ///              1 - (default level) Pointers to IFO files and some data from
    ///                  PrimaryVolumeDescriptor are cached.
    ///
    /// @return The level of caching.
    pub fn udf_cache_level(&mut self, cache_level: i32) -> i32 {
        unsafe {
            DVDUDFCacheLevel(
                &mut self.reader,
                cache_level,
            )
        }
    }

    /// Looks for a file on the UDF disc/imagefile and returns the block number
    /// where it begins, or 0 if it is not found.  The filename should be an
    /// absolute pathname on the UDF filesystem, starting with '/'.  For example,
    /// '/VIDEO_TS/VTS_01_1.IFO'.  On success, filesize will be set to the size of
    /// the file in bytes.
    pub fn udf_find_file(&mut self, path: &String) -> Result<(u32, u32), String> {
        let mut filesize: u32 = 0;
        match unsafe {
            UDFFindFile(
                &mut self.reader,
                path.as_ptr() as *const c_char,
                &mut filesize,
            )
        } {
            0 => {Err("File not found".to_string())},
            v => {Ok((v, filesize))},
        }
    }

    /// Gets the Volume Identifier string, in 8bit unicode (latin-1)
    /// volid, place to put the string
    /// volid_size, size of the buffer volid points to
    /// returns the size of buffer needed for all data
    pub fn udf_get_volume_identifier(&mut self) -> (i32, String){
        let mut buffer = [0u8; 32];
        let result = unsafe {
            UDFGetVolumeIdentifier(
                &mut self.reader,
                buffer.as_ptr() as *mut c_char,
                32,
            )
        };

        (
            result,
            String::from_utf8(buffer.to_vec()).unwrap(),
        )
    }

    /// Gets the Volume Set Identifier, as a 128-byte dstring (not decoded)
    /// WARNING This is not a null terminated string
    /// volsetid, place to put the data
    /// volsetid_size, size of the buffer volsetid points to
    /// the buffer should be >=128 bytes to store the whole volumesetidentifier
    /// returns the size of the available volsetid information (128)
    /// or 0 on error
    pub fn udf_get_volume_set_identifier(&mut self) -> (i32, String){
        let mut buffer = [0u8; 128];
        let result = unsafe {
            UDFGetVolumeSetIdentifier(
                &mut self.reader,
                buffer.as_mut_ptr(),
                128,
            )
        };

        (
            result,
            String::from_utf8(buffer.to_vec()).unwrap(),
        )
    }

    pub fn ifo_print(&mut self, title: i32) {
        unsafe {ifo_print(&mut self.reader, title)}
    }

    /// handle = ifoOpen(dvd, title);
    ///
    /// Opens an IFO and reads in all the data for the IFO file corresponding to the
    /// given title.  If title 0 is given, the video manager IFO file is read.
    /// Returns a handle to a completely parsed structure.
    pub fn ifo_open(&mut self, title: i32) -> ifo_handle_t{
        unsafe {
            *ifoOpen(
                &mut self.reader,
                title,
            )
        }
    }

    /// handle = ifoOpenVMGI(dvd);
    ///
    /// Opens an IFO and reads in _only_ the vmgi_mat data.  This call can be used
    /// together with the calls below to read in each segment of the IFO file on
    /// demand.
    pub fn ifo_open_vmgi(&mut self) -> ifo_handle_t {
        unsafe {
            *ifoOpenVMGI(&mut self.reader)
        }
    }

    /// handle = ifoOpenVTSI(dvd, title);
    ///
    /// Opens an IFO and reads in _only_ the vtsi_mat data.  This call can be used
    /// together with the calls below to read in each segment of the IFO file on
    /// demand.
    pub fn ifo_open_vtsi(&mut self, title: i32) -> ifo_handle_t {
        unsafe {
            *ifoOpenVTSI(
                &mut self.reader,
                title,
            )
        }
    }


}
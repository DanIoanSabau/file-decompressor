extern crate zip;

fn main() {
    let mut possibly_corupted_files = 0;
    let program_arguments = std::env::args().collect::<Vec<String>>();

    // check program arguments if they are correctly passed
    if 2 != program_arguments.len() {
        eprintln!("Usage: {} <compressed-file>", program_arguments[0]);
        return;
    }

    // getting the input zip file path from the program arguments
    let input_file_path = std::path::Path::new(&*program_arguments[1]);
    let input_file = match std::fs::File::open(&input_file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Cannot open the input file because of an error: {}.", err);
            return;
        }
    };

    // creating the zip archive using zip library
    let mut zip_archive = match zip::ZipArchive::new(input_file) {
        Ok(archive) => archive,
        Err(err) => {
            eprintln!("Cannot create the zip archive because of an error: {}.", err);
            return;
        }
    };

    // start the decompression process
    for file_number in 0..zip_archive.len() {
        let mut zip_file = match zip_archive.by_index(file_number) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Cannot open the file #{} from the zip archive because of an error: {}.", file_number, err);
                return;
            }
        };

        // make sure that the path is a valid path inside the current directory and not an absolute path 
        // else we mark that we've got a possible attack and continue with the next file inside the archive
        let output_file_path_buffer = match zip_file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => {
                possibly_corupted_files += 1;
                continue;
            }
        };

        // printing file's comment if exists
        {
            let file_comment = zip_file.comment();
            if !file_comment.is_empty() {
                println!("Commment for file #{}: {}", file_number, file_comment);
            }
        }

        // check if the file is a directory and create everything that is necessary, like parent directories & so on
        if (*zip_file.name()).ends_with('/') {
            println!("File #{} extracted to \"{}\".", file_number, output_file_path_buffer.display());
            if let Err(error) = std::fs::create_dir_all(&output_file_path_buffer) {
                eprintln!("Cannot create the directory \"{}\" because of an error: {}.", output_file_path_buffer.display(), error);
            }
        } else {
            println!("File #{} extracted to \"{}\" ({} bytes).", file_number, output_file_path_buffer.display(), zip_file.size());

            // make sure that the parent directory exists
            if let Some(parent_path) = output_file_path_buffer.parent() {
                if !parent_path.exists() {
                    if let Err(error) = std::fs::create_dir_all(&parent_path) {
                        eprintln!("Cannot create the file's \"{}\" parent directory because of an error: {}.", parent_path.display(), error);
                    }
                }
            }

            // create the output file
            let mut output_file = match std::fs::File::create(&output_file_path_buffer) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("Cannot create the output file \"{}\" because of an error: {}.", output_file_path_buffer.display(), err);
                    return;
                }
            };

            // copy the input to the output file
            if let Err(error) = std::io::copy(&mut zip_file, &mut output_file) {
                eprintln!("Cannot write the zip file \"{}\" because of an error: {}.", output_file_path_buffer.display(), error);
            };
        }

        // set the neccessary permissions if we're on a unix machine for user to have access to the files
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = zip_file.unix_mode() {
                if let Err(error) = std::fs::set_permissions(&output_file_path_buffer, std::fs::Permissions::from_mode(mode)) {
                    eprintln!("Cannot set the permissions of the output file \"{}\" because of an error: {}.", output_file_path_buffer.display(), error);
                }
            }
        }
    }

    println!("Possibly corupted files encountered during the decompression process: {}", possibly_corupted_files);
}

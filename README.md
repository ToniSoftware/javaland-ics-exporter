# JavaLand ICS Exporter

Just a quick and dirty tool to export the JavaLand 2023 Agenda to .ics-Files.

To run it, make sure that you have the folders `20, 21, 22 `and `23` in the root dir.
The script will place the finished .ics-Files corresponding to the date into the folders.
Afterwards simply run `cargo run` and there you go.
The files contain the title, speaker and location.

If you want all events to be saved within one .ics-File simply run `cargo run -o true`

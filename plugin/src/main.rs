use aid_plugin::AudioInterfaceDiag;
use nih_plug::prelude::*;

fn main() {
    nih_export_standalone::<AudioInterfaceDiag>();
}

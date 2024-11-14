#[macro_export]
macro_rules! get_handle_output {
    (noerr) => {
        match TERMINAL {
            Some(ref ter) => match ter.lock() {
                Ok(ter) => ter.output.clone(),
                Err(_) => panic!("Couldnt get Output Handle"),
            },
            None => panic!("Couldnt get Output Handle"),
        }
    };
    () => {
        match TERMINAL {
            Some(ref ter) => match ter.lock() {
                Ok(ter) => Ok(ter.output.clone()),
                Err(_) => Err(()),
            },
            None => Err(()),
        }?
    };
}
#[macro_export]
macro_rules! get_handle_input {
    (noerr) => {
        match TERMINAL {
            Some(ref ter) => match ter.lock() {
                Ok(ter) => ter.input.clone(),
                Err(_) => panic!("Couldnt get Output Handle"),
            },
            None => panic!("Terminal not initialized"),
        }
    };
    () => {
        match TERMINAL {
            Some(ref ter) => match ter.lock() {
                Ok(ter) => Ok(ter.input.clone()),
                Err(_) => Err(()),
            },
            None => Err(()),
        }?
    };
}
#[macro_export]
macro_rules! get_attr {
    () => {
        match TERMINAL {
            Some(ref ter) => match ter.lock().unwrap() {
                &ter.attr
            },
            None => panic!("Terminal not initialized"),
        }?
    };
}

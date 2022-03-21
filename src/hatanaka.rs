//! hatanaka.rs   
//! structures and macros suites for
//! RINEX OBS files compression and decompression.   
use crate::header;
use crate::record;
use crate::record::Sv;
use thiserror::Error;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Error, Debug)]
/// Hatanaka Kernel, compression
/// and decompression related errors
pub enum KernelError {
    #[error("order cannot be greater than {0}")]
    OrderTooBig(usize),
    #[error("cannot recover a different type than init type!")]
    TypeMismatch,
}

/// `Dtype` describes the type of data
/// we can compress / decompress
#[derive(Clone, Debug)]
enum Dtype {
    /// Numerical is intended to be used    
    /// for observation data and clock offsets
    Numerical(i64),
    /// Text represents epoch descriptors,   
    /// GNSS descriptors,    
    /// observation flags..
    Text(String),
}

impl Default for Dtype {
    fn default() -> Dtype { Dtype::Numerical(0) }
 }

impl Dtype {
    pub fn as_numerical (&self) -> Option<i64> {
        match self {
            Dtype::Numerical(n) => Some(n.clone()),
            _ => None,
        }
    }
    pub fn as_text (&self) -> Option<String> {
        match self {
            Dtype::Text(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

/// `Kernel` is a structure to compress    
/// or recover data using recursive defferential     
/// equations as defined by Y. Hatanaka.   
/// No compression limitations but maximal order   
/// to be supported must be defined on structure     
/// creation for memory allocation efficiency.
pub struct Kernel {
    /// internal counter
    n: usize,
    /// compression order
    order: usize,
    /// kernel initializer
    init: Dtype,
    /// state vector 
    state: Vec<i64>,
    /// previous state vector 
    p_state: Vec<i64>,
}

/// Fills given i64 vector with '0'
fn zeros (array : &mut Vec<i64>) {
    for i in 0..array.len() { array[i] = 0 }
}

impl Kernel {
    /// Builds a new kernel structure.    
    /// m: maximal Hatanaka order for this kernel to ever support,    
    ///    m=5 is hardcoded in CRN2RNX official tool
    fn new (m: usize) -> Kernel {
        let mut state : Vec<i64> = Vec::with_capacity(m+1);
        for _ in 0..m+1 { state.push(0) }
        Kernel {
            n: 0,
            order: 0,
            init: Dtype::default(),
            state: state.clone(),
            p_state: state.clone(),
        }
    }

    /// Initializes kernel.   
    /// order: compression order   
    /// data: kernel initializer
    fn init (&mut self, order: usize, data: Dtype) -> Result<(), KernelError> { 
        if order > self.state.len() {
            return Err(KernelError::OrderTooBig(self.state.len()))
        }
        // reset
        self.n = 0;
        zeros(&mut self.state);
        zeros(&mut self.p_state);
        // init
        self.init = data.clone();
        self.p_state[0] = data.as_numerical().unwrap_or(0);
        self.order = order;
        Ok(())
    }

    /// Recovers data by computing
    /// recursive differential equation
    fn recover (&mut self, data: Dtype) -> Result<Dtype, KernelError> {
        match data {
            Dtype::Numerical(data) => {
                if let Some(_) = self.init.as_numerical() {
                    Ok(self.numerical_data_recovery(data))
                } else {
                    Err(KernelError::TypeMismatch)
                }
            },
            Dtype::Text(data) => {
                if let Some(_) = self.init.as_text() {
                   Ok(self.text_data_recovery(data))
                } else {
                    Err(KernelError::TypeMismatch)
                }
            },
        }
    }
    /// Runs Differential Equation
    /// as defined in Hatanaka compression method
    fn numerical_data_recovery (&mut self, data: i64) -> Dtype {
        self.n += 1;
        self.n = std::cmp::min(self.n, self.order);
        zeros(&mut self.state);
        self.state[self.n] = data;
        for index in (0..self.n).rev() {
            self.state[index] = 
                self.state[index+1] 
                    + self.p_state[index] 
        }
        self.p_state = self.state.clone();
        Dtype::Numerical(self.state[0])
    }
    /// Text is very simple
    fn text_data_recovery (&mut self, data: String) -> Dtype {
        let mut init = self.init
            .as_text()
            .unwrap();
        let l = init.len();
        let mut recovered = String::from("");
        let mut p = init.as_mut_str().chars();
        let mut data = data.as_str().chars();
        for _ in 0..l {
            let next_c = p.next().unwrap();
            if let Some(c) = data.next() {
                if c == '&' {
                    recovered.push_str(" ")
                } else if c.is_ascii_alphanumeric() {
                    recovered.push_str(&c.to_string())
                } else {
                    recovered.push_str(&next_c.to_string())
                }
            } else {
                recovered.push_str(&next_c.to_string())
            }
        }
        // mask might be longer than self
        // in case we need to extend current value
        loop {
            if let Some(c) = data.next() {
                if c == '&' {
                    recovered.push_str(" ")
                } else if c.is_ascii_alphanumeric() {
                    recovered.push_str(&c.to_string())
                }
            } else {
                break
            }
        }
        self.init = Dtype::Text(recovered.clone()); // for next time
        Dtype::Text(String::from(&recovered))
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("kernel error")]
    KernelError(#[from] KernelError),
    #[error("invalid crinex format")]
    CrinexFormatError,
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse sat. vehicule")]
    ParseSvError(#[from] record::ParseSvError), 
}

/// `Hatanaka` structure can compress   
/// or decompress entire epochs inside a RINEX record    
/// using `Hatanaka` algorithm
pub struct Hatanaka {
    pub epo_krn_init: bool,
    pub epo_krn: Kernel,
    pub clk_krn: Kernel,
    pub obs_krn: HashMap<Sv, HashMap<String, (Kernel,Kernel,Kernel)>>,
}

impl Hatanaka {
    /// Creates a new `Hatanaka` core 
    /// m : maximal compression / decompression order
    pub fn new (m: usize) -> Hatanaka {
        Hatanaka {
            epo_krn_init: true,
            epo_krn : Kernel::new(m),
            clk_krn : Kernel::new(m),
            obs_krn : HashMap::new(),
        }
    }
}

/// Decompresses entire block content from a record.   
/// header: previous parsed `RinexHeader`   
/// core : decompression `Hatanaka` core    
/// content: entire epoch content    
/// Returns recovered epoch content
pub fn decompress (header: &header::RinexHeader, core: &mut Hatanaka, content: &str)
        -> Result<String, Error> {
/* TODO
    1) manage comments:
       always preserved
       manage weird case where line#1 is a comment,
         although should never happen
  
    2) e_flag > 2 must be preserved !
        revoir dans la publi, l'exemple ou ca arrive
*/
    let mut result = String::new();
    let mut lines = content.lines();
    let mut line = lines.next()
        .unwrap();
    let crinex = header.crinex
        .as_ref()
        .unwrap();
    if core.epo_krn_init {
        match crinex.version.major { // sanity checks
            1 | 2 => { // old CRINEX
                if !line.starts_with("&") {
                    return Err(Error::CrinexFormatError)
                }
            },
            _ => { // modern CRINEX
                if !line.starts_with("> ") {
                    return Err(Error::CrinexFormatError)
                }
            },
        }
        // init epoch kernel
        core.epo_krn.init(0, Dtype::Text(line.to_string()))
            .unwrap();
    }
    let recover = core.epo_krn.recover(Dtype::Text(line.to_string()))?;
    let recovered_epoch = recover.as_text()
        .unwrap();

    let obs_codes = header.obs_codes
        .as_ref()
        .unwrap();
    // recovered epoch description
    //  * identify epoch flag for further logic
    //  * build OBS/flags kernel (3 per Sv)
    let e_str = recovered_epoch.as_str(); 
    let mut offset : usize =
        2     // Y
        +2+1 // m
        +2+1 // d
        +2+1 // h
        +2+1 // m
        +11  // s
        +1;  // ">" or "&"
    if header.version.major > 2 {
        offset += 2 // Y is 4 digit
    }
    if crinex.version.major > 2 { // CRINEX3
        offset += 1 // has 1 extra whitespace
    }
    let (_, rem) = e_str.split_at(offset); // (epoch, )
    let (e_flag, rem) = rem.split_at(3); // (flag, )
    let (e_len, rem) = rem.split_at(3); // (len, )
    let e_flag = u8::from_str_radix(e_flag.trim(), 10)?;
    let e_len = u8::from_str_radix(e_len.trim(), 10)?;

    let (_, rem) = rem.split_at(7);
    for i in 0..e_len {
         let (sv, rem) = rem.split_at(3);
         let sv = Sv::from_str(sv)?;
         if !core.obs_krn.contains_key(&sv) { 
            // sat system, never seen before
            let codes = &obs_codes[&sv.constellation];
            // ==> create new kernel entries
            //     one per OBS code
            //     with same compression capacity as other kernels
            let mut map : HashMap<String, (Kernel,Kernel,Kernel)> = HashMap::with_capacity(16);
            for code in codes {
                let kernels = ( 
                    Kernel::new(core.epo_krn.state.len()-1),
                    Kernel::new(core.epo_krn.state.len()-1),
                    Kernel::new(core.epo_krn.state.len()-1),
                );
                map.insert(code.to_string(), kernels); 
            }
            core.obs_krn.insert(sv, map);
         }
    }

    // clock offset
    line = lines.next().unwrap();
    let clock_offset : Option<i64> = match line.contains("&") { 
        true => {
            // kernel (re)init
            let (n, rem) = line.split_at(1);
            let n = u8::from_str_radix(n, 10)?;
            let (_, num) = rem.split_at(1);
            let num = i64::from_str_radix(num, 10)?;
            core.clk_krn.init(n.into(), Dtype::Numerical(num))
                .unwrap();
            Some(num)
        },
        false => {
            // clock offset is optionnal
            if let Ok(num) = i64::from_str_radix(line.trim(), 10) {
                let recovered = core.clk_krn.recover(Dtype::Numerical(num))
                    .unwrap();
                Some(recovered.as_numerical().unwrap())
            } else {
                None
            }
        }
    };

    // recover epoch descriptor
    /*let epoch = recovered_epoch.as_str();
    let epoch = epoch.split_at(35).0;
    match header.version.major {
        1 | 2 => { // Old RINEX
            // TODO
            // il faut wrapper tous les sat de cette epoch
            // sur autant de ligne que necessaire,
            // revoir dans src/observation.rs 
            // le nb de Sv permis par lignes
            // TODO
            // clock offset est a squeezer en ligne #1
        },
        _ => { // Modern RINEX
            result.push_str(epoch);
            if let Some(offset) = clock_offset {
                result.push_str(&format!("          {}\n", (offset as f64)/1000.0_f64))
            }
        },
    }*/
     
    // recover epoch content
    /*let sv_index : usize = 0; // to index Sv
    for line in lines.next() {
        let mut content : String;
        match header.version.major {
            1 | 2 => { // Old RINEX
            },
            _ => { // Modern RINEX
            },
        }
        result.push_str(&format!("{}\n", content)) //TODO ce \n en derniere ligne va faire chier par la suite
    }*/
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    /// Tests numerical data recovery    
    /// through Hatanaka decompression.   
    /// Tests data come from an official CRINEX file and
    /// official CRX2RNX decompression tool
    fn test_num_recovery() {
        let mut krn = Kernel::new(5);
        let init : i64 = 25065408994;
        krn.init(3, Dtype::Numerical(init))
            .unwrap();
        let data : Vec<i64> = vec![
            5918760,
            92440,
            -240,
            -320,
            -160,
            -580,
            360,
            -1380,
            220,
            -140,
        ];
        let expected : Vec<i64> = vec![
            25071327754,
            25077338954,
            25083442354,
            25089637634,
            25095924634,
            25102302774,
            25108772414,
            25115332174,
            25121982274,
            25128722574,
        ];
        for i in 0..data.len() {
            let recovered = krn.recover(Dtype::Numerical(data[i]))
                .unwrap()
                    .as_numerical()
                    .unwrap();
            assert_eq!(recovered, expected[i]);
        }
        // test re-init
        let init : i64 = 24701300559;
        krn.init(3, Dtype::Numerical(init))
            .unwrap();
        let data : Vec<i64> = vec![
            -19542118,
            29235,
            -38,
            1592,
            -931,
            645,
            1001,
            -1038,
            2198,
            -2679,
            2804,
            -892,
        ];
        let expected : Vec<i64> = vec![
            24681758441,
            24662245558,
            24642761872,
            24623308975,
            24603885936,
            24584493400,
            24565132368,
            24545801802,
            24526503900,
            24507235983,
            24488000855,
            24468797624,
        ];
        for i in 0..data.len() {
            let recovered = krn.recover(Dtype::Numerical(data[i]))
                .unwrap()
                    .as_numerical()
                    .unwrap();
            assert_eq!(recovered, expected[i]);
        }
    }
    #[test]
    /// Tests Hatanaka Text Recovery algorithm
    fn test_text_recovery() {
        let init = "ABCDEFG 12 000 33 XXACQmpLf";
        let mut krn = Kernel::new(5);
        let masks : Vec<&str> = vec![
            "        13   1 44 xxACq   F",
            " 11 22   x   0 4  y     p  ",
            "              1     ",
            "                   z",
            " ",
        ];
        let expected : Vec<&str> = vec![
            "ABCDEFG 13 001 44 xxACqmpLF",
            "A11D22G 1x 000 44 yxACqmpLF",
            "A11D22G 1x 000144 yxACqmpLF",
            "A11D22G 1x 000144 yzACqmpLF",
            "A11D22G 1x 000144 yzACqmpLF",
        ];
        krn.init(3, Dtype::Text(String::from(init)))
            .unwrap();
        for i in 0..masks.len() {
            let mask = masks[i];
            let result = krn.recover(Dtype::Text(String::from(mask)))
                .unwrap()
                    .as_text()
                    .unwrap();
            assert_eq!(result, String::from(expected[i]));
        }
        // test re-init
        let init = " 2200 123      G 07G08G09G   XX XX";
        krn.init(3, Dtype::Text(String::from(init)))
            .unwrap();
        let masks : Vec<&str> = vec![
            "        F       1  3",
            " x    1 f  f   p",
            " ",
            "  3       4       ",
        ];
        let expected : Vec<&str> = vec![
            " 2200 12F      G107308G09G   XX XX",
            " x200 12f  f   p107308G09G   XX XX",
            " x200 12f  f   p107308G09G   XX XX",
            " x300 12f 4f   p107308G09G   XX XX",
        ];
        for i in 0..masks.len() {
            let mask = masks[i];
            let result = krn.recover(Dtype::Text(String::from(mask)))
                .unwrap()
                    .as_text()
                    .unwrap();
            assert_eq!(result, String::from(expected[i]));
        }
    }
}

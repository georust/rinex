#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::header::Header;
    use rinex::version::Version;
    use rinex::hatanaka::Hatanaka;
    use rinex::observation::Crinex;
    use rinex::observation::HeaderFields;
    use rinex::constellation::Constellation;
    use std::collections::HashMap;
    #[test]
    fn test_rnx_decompression() {
        // object
        let mut decompressor = Hatanaka::new(8);
        // fake header
        let mut header = Header::basic_obs();
        header.obs = Some(observation::HeaderFields {
            crinex: Some(
                Crinex {
                    version: Version { 
                        minor: 0,
                        major: 1,
                    },
                    prog: String::from("Test"),
                    date: chrono::Utc::now().naive_utc(),
                }
            ),
            codes: {
                let mut codes: HashMap<Constellation, Vec<String>> = HashMap::new();
                codes.insert(
                    Constellation::GPS,
                    vec![
                        String::from("L1"),
                        String::from("L2"),
                        String::from("C1"),
                        String::from("P2"),
                        String::from("P1"),
                        String::from("S1"),
                        String::from("S2")
                    ]);
                codes.insert(
                    Constellation::Glonass,
                    vec![
                        String::from("L1"),
                        String::from("L2"),
                        String::from("C1"),
                        String::from("P2"),
                        String::from("P1"),
                        String::from("S1"),
                        String::from("S2")
                    ]);
                codes
            },
            clock_offset_applied: false,
        });
        // real CRX data [epoch #1]
        let content = "&21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16R18G13R01R16R17G15R02R15";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        println!("RESULT\n\"{}\"", result);

        let content = "";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        println!("RESULT\n\"{}\"", result);
        
        let content = "3&126298057858 3&98414080647 3&24033720416 3&24033721351 3&24033719353 3&40000 3&22000  643        4";
        let decompressed = decompressor.decompress(&header, content); 
        let result = decompressed.unwrap();
        println!("RESULT\n\"{}\"", result);
        
        /*
        //assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&111982965979 3&87259475177 3&21309646971 3&21309649924 3&21309646771 3&48000 3&37000  846        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&125184221815 3&97546158899 3&23821762469 3&23821768749 3&23821762040 3&40000 3&35000  645        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&111582858305 3&86947705907 3&21233509912 3&21233511727 3&21233508599 3&49000 3&37000  846        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&123948324318 3&96583131804 3&23586581658 3&23586584420 3&23586580391 3&39000 3&20000  643        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&121443586995 3&94631394735 3&23109944474 3&23109947055 3&23109944139 3&40000 3&28000  644        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&123664246260 3&96183328899 3&23125836575 3&23125839071 3&23125836244 3&41000 3&40000  6 6";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&120184930156 3&93477190884 3&22506776986 3&22506777899 3&22506777465 3&44000 3&43000  7 7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&114160130658 3&88955964556 3&21723948105 3&21723953153 3&21723947155 3&46000 3&47000  747        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&105271498527 3&82029766966 3&20032495677 3&20032500289 3&20032495191 3&53000 3&56000  849        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&112144051840 3&87384999714 3&21340302567 3&21340307619 3&21340301864 3&52000 3&52000  848        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&113556669792 3&88485725931 3&21609114743 3&21609116936 3&21609113593 3&47000 3&38000  746        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&106844822639 3&83101546155 3&20015628375 3&20015631390 3&20015628486 3&53000 3&50000  8 8";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&131399268954 3&102389055312 3&25004448492 3&25004450593 3&25004447809 3&36000 3&12000  642        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&116954721816 3&90964809026 3&21878794287 3&21878798089 3&21878794473 3&42000 3&39000  7 6";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&110150054052 3&85672270182 3&20620317569 3&20620319227 3&20620317319 3&46000 3&42000  7 7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&104344364269 3&81156754225 3&19499235584 3&19499235915 3&19499235267 3&52000 3&51000  8 8";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&126812563577 3&98815006750 3&24131624962 3&24131627813 3&24131624907 3&38000 3&29000  644        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&113907865790 3&88595032353 3&21346285819 3&21346288923 3&21346285443 3&49000 3&46000  8 7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "3&118516772306 3&92179732837 3&22178802374 3&22178804901 3&22178802684 3&45000 3&42000  7 7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "                3";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "-15603288 -12158423 -2969836 -2968829 -2968864 -1000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);*/
    }
}

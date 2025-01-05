// continous V3 data extracted from VLNS0630 compressed by
// RNX2CRX and decompressed with CRX2RNX historical tools
use crate::tests::crinex::decompression::run_raw_decompression_test;

const INPUT : &str = "> 2022 03 04  0  0  0.0000000  0 22      G01G03G04G06G09G12G17G19G21G22G25G31G32R01R02R07R08R09R10R17R23R24
3&0
3&20832393682 3&109474991854 3&49500  3&20832389822     3&85305196437     3&49500    &&&8&&&&&&&&&&&&&&&8&&&&&&&&&&&&&&&&
3&20342516786 3&106900663487 3&50000  3&20342512006     3&83299201382     3&50000    &&&8&&&&&&&&&&&&&&&8&&&&&&&&&&&&&&&&
3&22448754952 3&117969025322 3&48250  3&22448749312     3&91923884833     3&43750    &&&8&&&&&&&&&&&&&&&7&&&&&&&&&&&&&&&&
3&24827263216 3&130468159526 3&39750  3&24827259316     3&101663482505     3&37250    &&&6&&&&&&&&&&&&&&&6&&&&&&&&&&&&&&&&
3&25493930890 3&133971510403 3&41250  3&25493926950     3&104393373997     3&41750    &&&6&&&&&&&&&&&&&&&6&&&&&&&&&&&&&&&&
3&24938100714 3&131050615416 3&42000  3&24938094154     3&102117358264     3&39750    &&&7&&&&&&&&&&&&&&&6&&&&&&&&&&&&&&&&
3&22637157190 3&118959082996 3&48750  3&22637149810     3&92695365319     3&44250    &&&8&&&&&&&&&&&&&&&7&&&&&&&&&&&&&&&&
3&22621085930 3&118874641047 3&48000  3&22621076410     3&92629565853     3&44250    &&&8&&&&&&&&&&&&&&&7&&&&&&&&&&&&&&&&
3&22981954788 3&120771005265 3&48500  3&22981947428     3&94107267205     3&42750    &&&8&&&&&&&&&&&&&&&7&&&&&&&&&&&&&&&&
3&22622086950 3&118879893721 3&48000  3&22622079610     3&92633668221     3&40500    &&&8&&&&&&&&&&&&&&&6&&&&&&&&&&&&&&&&
3&24977789014 3&131259177970 3&41750  3&24977784474     3&102279837232     3&41750    &&&6&&&&&&&&&&&&&&&6&&&&&&&&&&&&&&&&
3&22153988434 3&116420013780 3&49000  3&22153980814     3&90716866768     3&49000    &&&8&&&&&&&&&&&&&&&8&&&&&&&&&&&&&&&&
3&24045998320 3&126362595965 3&44500  3&24045992840     3&98464353498     3&44500    &&&7&&&&&&&&&&&&&&&7&&&&&&&&&&&&&&&&
3&19592643312 3&104733904039 3&46750  3&19592644332  3&81459723978  3&42500 &&&7&&&&&&&&&7&&&&
3&23736040204 3&126660054136 3&45000  3&23736040864  3&98513387249  3&39750 &&&7&&&&&&&&&6&&&&
3&23599966724 3&126332490591 3&43750  3&23599965064  3&98258586223  3&40500 &&&7&&&&&&&&&6&&&&
3&19950172928 3&106832264599 3&46750  3&19950170508  3&83091782675  3&47000 &&&7&&&&&&&&&7&&&&
3&22347623252 3&119335090339 3&46500  3&22347621152  3&92816181896  3&43500 &&&7&&&&&&&&&7&&&&
3&22976124428 3&122475679549 3&42000       &&&7&&&&&&&&&&&&&&
3&21721839376 3&116237973201 3&47250  3&21721837656  3&90407313143  3&44000 &&&7&&&&&&&&&7&&&&
3&21159412480 3&113188605748 3&47000       &&&7&&&&&&&&&&&&&&
3&19646230552 3&105057234974 3&49250  3&19646229132  3&81711176383  3&46750 &&&8&&&&&&&&&7&&&&
                   3
0
12950080 68053048 250  12950100     53028324     250
-5366480 -28201082 250  -5366480     -21974880     0
-18019160 -94691404 0  -18019180     -73785490     -1000
-20607260 -108291727 500  -20607260     -84383119     1750
-21758358 -114341066 3500  -21758358     -89096965     1000       7               7
-159280 -837125 250  -159300     -652313     -1750
3508840 18439002 250  3508840     14368044     500
-4553260 -23927645 250  -4553240     -18644947     -1250
16409080 86230203 0  16409100     67192348     -1000                       6
14779680 77667666 250  14779700     60520245     -1000
-8946880 -47014687 -2500  -8946580     -36634844     -3500
-4052840 -21297718 0  -4052860     -16595611     0
19021920 99961103 2250  19021940     77891785     2000
-8232880 -44009323 0  -8232860  -34229459  -500
-21512940 -114796978 1500  -21512920  -89286524  750
21744180 116398218 750  21744200  90531903  -250
11675900 62523907 -500  11675900  48629697  -250
-2582540 -13790671 1000  -2582560  -10726077  -1000
-18855120 -100507691 2000
-19340540 -103495155 500  -19340540  -80496232  -500
21437600 114676771 500
2797000 14956818 250  2797020  11633067  -250";

const OUTPUT:  &str = "
> 2022 03 04  0  0  0.0000000  0 22       0.000000000000
G01  20832393.682   109474991.854 8        49.500                    20832389.822                                                                    85305196.437 8                                                                        49.500
G03  20342516.786   106900663.487 8        50.000                    20342512.006                                                                    83299201.382 8                                                                        50.000
G04  22448754.952   117969025.322 8        48.250                    22448749.312                                                                    91923884.833 7                                                                        43.750
G06  24827263.216   130468159.526 6        39.750                    24827259.316                                                                   101663482.505 6                                                                        37.250
G09  25493930.890   133971510.403 6        41.250                    25493926.950                                                                   104393373.997 6                                                                        41.750
G12  24938100.714   131050615.416 7        42.000                    24938094.154                                                                   102117358.264 6                                                                        39.750
G17  22637157.190   118959082.996 8        48.750                    22637149.810                                                                    92695365.319 7                                                                        44.250
G19  22621085.930   118874641.047 8        48.000                    22621076.410                                                                    92629565.853 7                                                                        44.250
G21  22981954.788   120771005.265 8        48.500                    22981947.428                                                                    94107267.205 7                                                                        42.750
G22  22622086.950   118879893.721 8        48.000                    22622079.610                                                                    92633668.221 6                                                                        40.500
G25  24977789.014   131259177.970 6        41.750                    24977784.474                                                                   102279837.232 6                                                                        41.750
G31  22153988.434   116420013.780 8        49.000                    22153980.814                                                                    90716866.768 8                                                                        49.000
G32  24045998.320   126362595.965 7        44.500                    24045992.840                                                                    98464353.498 7                                                                        44.500
R01  19592643.312   104733904.039 7        46.750                    19592644.332                    81459723.978 7                        42.500
R02  23736040.204   126660054.136 7        45.000                    23736040.864                    98513387.249 6                        39.750
R07  23599966.724   126332490.591 7        43.750                    23599965.064                    98258586.223 6                        40.500
R08  19950172.928   106832264.599 7        46.750                    19950170.508                    83091782.675 7                        47.000
R09  22347623.252   119335090.339 7        46.500                    22347621.152                    92816181.896 7                        43.500
R10  22976124.428   122475679.549 7        42.000
R17  21721839.376   116237973.201 7        47.250                    21721837.656                    90407313.143 7                        44.000
R23  21159412.480   113188605.748 7        47.000
R24  19646230.552   105057234.974 8        49.250                    19646229.132                    81711176.383 7                        46.750

> 2022 03 04  0  0 30.0000000  0 22       0.000000000000
G01  20845343.762   109543044.902 8        49.750                    20845339.922                                                                    85358224.761 8                                                                        49.750
G03  20337150.306   106872462.405 8        50.250                    20337145.526                                                                    83277226.502 8                                                                        50.000
G04  22430735.792   117874333.918 8        48.250                    22430730.132                                                                    91850099.343 7                                                                        42.750
G06  24806655.956   130359867.799 6        40.250                    24806652.056                                                                   101579099.386 6                                                                        39.000
G09  25472172.532   133857169.337 7        44.750                    25472168.592                                                                   104304277.032 7                                                                        42.750
G12  24937941.434   131049778.291 7        42.250                    24937934.854                                                                   102116705.951 6                                                                        38.000
G17  22640666.030   118977521.998 8        49.000                    22640658.650                                                                    92709733.363 7                                                                        44.750
G19  22616532.670   118850713.402 8        48.250                    22616523.170                                                                    92610920.906 7                                                                        43.000
G21  22998363.868   120857235.468 8        48.500                    22998356.528                                                                    94174459.553 6                                                                        41.750
G22  22636866.630   118957561.387 8        48.250                    22636859.310                                                                    92694188.466 6                                                                        39.500
G25  24968842.134   131212163.283 6        39.250                    24968837.894                                                                   102243202.388 6                                                                        38.250
G31  22149935.594   116398716.062 8        49.000                    22149927.954                                                                    90700271.157 8                                                                        49.000
G32  24065020.240   126462557.068 7        46.750                    24065014.780                                                                    98542245.283 7                                                                        46.500
R01  19584410.432   104689894.716 7        46.750                    19584411.472                    81425494.519 7                        42.000
R02  23714527.264   126545257.158 7        46.500                    23714527.944                    98424100.725 6                        40.500
R07  23621710.904   126448888.809 7        44.500                    23621709.264                    98349118.126 6                        40.250
R08  19961848.828   106894788.506 7        46.250                    19961846.408                    83140412.372 7                        46.750
R09  22345040.712   119321299.668 7        47.500                    22345038.592                    92805455.819 7                        42.500
R10  22957269.308   122375171.858 7        44.000
R17  21702498.836   116134478.046 7        47.750                    21702497.116                    90326816.911 7                        43.500
R23  21180850.080   113303282.519 7        47.500
R24  19649027.552   105072191.792 8        49.500                    19649026.152                    81722809.450 7                        46.500";

#[test]
fn v3_vlns0630() {
    run_raw_decompression_test(
        true,
        "MIXED",
        &["GPS", "GLO"],
        &[
            "C1C, L1C, S1C, C2P, C2W, C2S, C2L, C2X, L2P, L2W, L2S, L2L, L2X, S2P, S2W, S2S, S2L, S2X",
            "C1C, L1C, S1C, C2C, C2P, L2C, L2P, S2C, S2P",
        ],
        INPUT,
        OUTPUT,
    );
}
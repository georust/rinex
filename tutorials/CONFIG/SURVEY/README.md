(Static) Surveying
==================

All this custom configuration unlock the post processed navigation solutions
and will have them auto-integrated to the Qc analysis.

This is done by the `ppp` and/or `cggtts` field of the `solutions` option:

Example: unlock PPP (Post Processed Positioning) (only)

```json
{
    "solutions": {
        "ppp": true
    }
}
```

Example: unlock CGGTTS special PPP - timing oriented solutions (only)

```json
{
    "solutions": {
        "cggtts": true
    }
}
```

Example: unlock both type of solutions

```json
{
    "solutions": {
        "ppp": true,
        "cggtts": true
    }
}
```

All following configurations:

1. `ppp_`: means that only "ppp" is enabled
2. `cggtts_`: means that only "cggtts" option is enabled.
3. We don't have examples that use both, because all this tutorial serie is splitted by specific topic
4. `xxx_kf`: the following term refers to the navigation filter being used

## Solver options

PPP solution solving is quite advanced and already out of scope of pure RINEX / SP3 processing. 

The PPP solver options are documented in [the RTK-RS library]().

## CGGTTS

CGGTTS involves both the RTK-RS solver and the CGGTTS track scheduler. Both are different things
and have their own set of options.

Therefore, CGGTTS configuration is one of the most advanced configuration setup, because

1. it contains the Qc configuration
2. it allows tweaking the RTK solver (navigation solver)
3. it allows customizing the CGGTTS tracker 

Since CGGTTS implicitely means static application, it is the only case that truly
requires the definition of a static location (x, y, z) on Earth.

To do so, the toolbox offers several options

1. Use RINEX files that describe the receiver location.
In our examples, this applies to `tutorials/V3/MONJDNK` or `tutorials/V3/ESBJRDNK` that
describes (in RINEX) the (x, y, z) coordinates of the GNSS receiver.
This is the most convenient and easiest option.

2. Use `rinex-cli` to [patch your RINEX and define (x, y, z) yourself]()

3. Define (x, y, z) through `Qc` configuration.
The coordinates defined in the `Qc` configuration always superceed the coordinates
we may pick up in the dataset. This may also cover the case where your RINEX were
geodetically surveyed, and you have an updated survey results (improved results), you can use
this option to prefer the updated results.

## PPP / CGGTTS and initial position knowledge

PPP does not require initial apriori knowlege. The toolbox is able to operate
in [full survey mode](), this scenario may apply to the designing and calibration of a new RTK reference
station.

Yet a few points need to be understood in this scenario:

1. disregarding your navigation technique and preference, the solver will require 4 vehicles
to be sampled and correctly sighted for a small initialization period. 
2. Therefore, operating without initial knowledge has more "stringent" initialization requirements
3. Operating with (x, y, z) apriori knowledge (either by RINEX definition, or manual definition)
means you can deploy your navigation technique and preference even on the first solver iteration.
4. CGGTTS solving will rapidly panic and not be able to deploy, without (x, y, z) knowledge, 
because CGTTTS implicitely means Time Only navigation technique (aka. single vehicle based navigation)
which requires (x, y, z) knowledge at all times.
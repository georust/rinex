ionex2kml
=========

`ionex2kml` is a small application to convert a IONEX file into KML format.

It is a convenient method to visualize TEC maps using third party tools.

Getting started
===============

Provide a IONEX file with `-i`:

```bash
    ionex2kml -i /tmp/ionex.txt
```

Input IONEX files do not have to follow naming conventions.

This tool will preserve the input file name by default, just changing the internal
format and file extension. In the previous example we would get `/tmp/ionex.kml`.

Define the output file yourself with `-o`:

```bash
    ionex2kml -i jjim12.12i -o /tmp/test.kml
```

Each Epoch is put in a dedicated KML folder.  

Equipotential TEC values
========================

When converting to KML, we round the TEC values and shrink it to N equipotential areas.  
In other words, the granularity on the TEC visualization you get is max|tec|/N where max|tec|
is the absolute maximal TEC value in given file, through all epochs and all altitudes.

Another way to see this, is N defines the number of separate color the color map will be able to represent.

Visualizing KML maps
====================

Google maps is one way to visualize a KML file.

KML content customization
=========================

Define a specific KML revision with `-v`

```bash
    ionex2kml -i jjim12.12i -v http://www.opengis.net/kml/2.2
```

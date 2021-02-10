# gpxanalyzer

 GPX analyzer (best distance/time legs) and utility tools (merge, info extractor).

 ## Usage

Display info about a GPX:

    gpxanalyzer info --file input.gpx

Analyse best legs:

    gpxanalyzer analyze --file input.gpx --time 60
    gpxanalyzer analyze --file input.gpx --distance 1000

Merge multiple GPX together:

    gpxanalyzer merge --files *.gpx --output merge.gpx


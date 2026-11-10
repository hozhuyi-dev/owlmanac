"""Mapping the Milky Way: the post, as a notebook. Locally it queries Gaia (or reads its cache in data/);
exported, it runs in the reader's browser and reads the same cache over HTTP."""

import marimo

__generated_with = "0.25.2"
app = marimo.App(width="medium", app_title="Mapping the Milky Way", css_file="../notebook.css")


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ## Introduction

    In April 2025, during our honeymoon trip to Hehuanshan, Taiwan, my wife and I had the rare chance to see a truly dark sky. The air was thin and cold, but above us stretched a dome of stars — sharp, steady, and countless. Constellations like Leo and Virgo were easy to spot, their bright stars marking the arrival of the spring sky.

    But one feature was missing: the Milky Way. April isn't the right season for its bright galactic core, which stays hidden below the horizon. Standing there, I wondered what the galaxy would look like if we could see all of it — every star, in every direction, mapped precisely.

    This post turns that question into a data-driven exploration using Gaia Data Release 3 (DR3). By querying and processing Gaia's precise astrometric measurements, we can reconstruct how the Milky Way appears from our vantage point.

    This page is a [marimo](https://marimo.io) notebook, and it is running in your browser: the Python below executes in WebAssembly, with nothing on a server. Cells are reactive, so when you move a slider, every cell that depends on it runs again. Expand any cell's code to see how its output was made.
    """)
    return


@app.cell
def _():
    import importlib
    import json
    import math
    import sys

    import altair as alt
    import marimo as mo
    import polars as pl

    alt.data_transformers.disable_max_rows()

    # In the browser (Pyodide) the notebook can't reach the Gaia archive, so it reads the cached results.
    IN_BROWSER = sys.platform == "emscripten"
    # Gaia queries are slow, so their results are cached in data/; set REFRESH to query the archive again.
    REFRESH = False

    # Owlmanac's chart colours, checked for contrast and colour-blind separation. Series pass on both the
    # site's dark (#2b2b2b) and light pages. Magnitude is "dusk", a muted turbo: deep indigo through plum,
    # brick and ochre to unbleached cream. Its lightness only rises, so more always reads brighter (turbo's
    # rises and falls, drawing false edges), and dense regions glow on the dark page, the site's default.
    SERIES = ["#4e7cb7", "#ba664c", "#0e9a94", "#b18334", "#9d6298", "#71934c"]
    DUSK = ["#48588b", "#836192", "#b76d79", "#d18968", "#daaa6f", "#e1ca94", "#eee4c7"]

    @alt.theme.register("owlmanac", enable=True)
    def owlmanac():
        """The site's font, a transparent background and its colours. No axis or text colours: marimo
        sets those for the page's theme, and a theme's would override them."""
        return {
            "config": {
                "background": "transparent",
                "font": "Inter Variable",
                "view": {"stroke": None},
                "axis": {"grid": False, "labelFontSize": 11, "titleFontSize": 12, "titleFontWeight": 500},
                "legend": {"labelFontSize": 11, "titleFontSize": 12, "titleFontWeight": 500},
                "range": {"category": SERIES, "ramp": DUSK, "heatmap": DUSK},
            }
        }

    # Star counts span up to six orders of magnitude: a log scale, blended through the theme's dusk ramp in
    # Lab, which keeps the shades between its colours muted.
    COUNTS = alt.Scale(type="log", interpolate="lab")

    return COUNTS, DUSK, IN_BROWSER, REFRESH, SERIES, alt, importlib, json, math, mo, pl


@app.cell
def _(IN_BROWSER, REFRESH, importlib, json, mo, pl):
    def local_only(module):
        """Imports a module that only local runs use, for querying Gaia. The browser downloads every
        package a notebook names in an import statement, so these are imported by name instead."""
        return importlib.import_module(module)

    async def cached(name, query):
        """The result of `query()`, a slow Gaia query, kept in data/<name>.json beside this notebook.
        JSON rather than parquet: in the browser, polars reads files only through pyarrow (10 MB more),
        while Python's json module is already there."""
        if IN_BROWSER:
            from pyodide.http import pyfetch

            response = await pyfetch(str(mo.notebook_location() / "data" / f"{name}.json"))
            return pl.DataFrame(json.loads(await response.string()))
        path = mo.notebook_dir() / "data" / f"{name}.json"
        if REFRESH or not path.exists():
            path.parent.mkdir(parents=True, exist_ok=True)
            query().write_json(path)
        return pl.read_json(path)

    def adql(query):
        """Runs an ADQL query on the Gaia archive's servers, and returns its result."""
        Gaia = local_only("astroquery.gaia").Gaia
        return pl.from_pandas(Gaia.launch_job_async(query).get_results().to_pandas())

    return adql, cached, local_only


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ## Processing Gaia DR3 Data

    Before visualizing the Milky Way, the first step is to get familiar with the Gaia DR3 dataset itself. Gaia, operated by the European Space Agency, continuously measures the positions, brightness, and motions of more than 1.4 billion stars with extraordinary precision. Each record is a snapshot of where a star is and how it moves through space — together forming a detailed map of our galaxy.

    The Gaia archive offers several ways in. The simplest is the query object, which retrieves a small, predefined subset of data. The cone search method is designed for spatial queries: you give a point on the sky and a radius, and Gaia returns all sources within that circle. For larger or more complex queries, `launch_job` submits a custom query written in ADQL (Astronomical Data Query Language), a dialect of SQL tailored for astronomical data.

    Because the catalog holds more than a billion records, queries run asynchronously on the archive's servers. Each query below ran there once; its result is cached in a small parquet file, which is what your browser reads.

    ### First Glimpse of the Milky Way

    To begin, I focused on a small region near the galactic center, where the Milky Way appears most densely packed. `astroquery.Gaia.query_object_async` retrieves all sources within a narrow 0.2° × 0.2° patch centered at (RA = 266.4°, Dec = −29°) — roughly the direction of Sagittarius A\*, our galaxy's core.
    """)
    return


@app.cell
async def _(cached, local_only, pl):
    GLIMPSE = {"ra": 266.4, "dec": -29.0, "width": 0.2}

    def query_glimpse():
        u = local_only("astropy.units")
        SkyCoord = local_only("astropy.coordinates").SkyCoord
        Gaia = local_only("astroquery.gaia").Gaia

        Gaia.MAIN_GAIA_TABLE = "gaiadr3.gaia_source"
        Gaia.ROW_LIMIT = 5000
        side = u.Quantity(GLIMPSE["width"], u.deg)
        centre = SkyCoord(ra=GLIMPSE["ra"], dec=GLIMPSE["dec"], unit=(u.deg, u.deg), frame="icrs")
        stars = pl.from_pandas(Gaia.query_object_async(coordinate=centre, width=side, height=side).to_pandas())
        return stars.select(pl.col("source_id").cast(pl.String), "ra", "dec", g_mag="phot_g_mean_mag").drop_nulls()

    glimpse = await cached("glimpse", query_glimpse)
    glimpse
    return GLIMPSE, glimpse


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    Each point below is a single star, positioned by its right ascension and declination. Marker size and colour show relative brightness, 21 − G, from Gaia's `phot_g_mean_mag`. Drag the slider to hide fainter stars.
    """)
    return


@app.cell
def _(mo):
    faintest = mo.ui.slider(10, 21.5, step=0.1, value=21.5, label="Faintest star shown, G ≤", show_value=True, full_width=True)
    return (faintest,)


@app.cell
def _(GLIMPSE, alt, faintest, glimpse, mo, pl):
    _half = GLIMPSE["width"] / 2
    _shown = (
        glimpse.filter(pl.col("g_mag") <= faintest.value)
        .with_columns(brightness=21 - pl.col("g_mag"))
        .sort("g_mag", descending=True)  # faint stars first, so bright ones draw on top
    )
    _chart = (
        alt.Chart(_shown)
        .mark_circle(opacity=0.85)
        .encode(
            x=alt.X(
                "ra:Q",
                title="Right ascension (°)",
                scale=alt.Scale(domain=[GLIMPSE["ra"] - _half, GLIMPSE["ra"] + _half], nice=False),
            ),
            y=alt.Y(
                "dec:Q",
                title="Declination (°)",
                scale=alt.Scale(domain=[GLIMPSE["dec"] - _half, GLIMPSE["dec"] + _half], nice=False),
            ),
            size=alt.Size("brightness:Q", scale=alt.Scale(domain=[0, 12], range=[2, 160], clamp=True), legend=None),
            # The theme's dusk ramp: faint stars small and deep, bright ones large and pale, as on a dark sky.
            color=alt.Color("brightness:Q", title="21 − G", scale=alt.Scale(domain=[0, 12], clamp=True)),
            tooltip=[
                alt.Tooltip("source_id:N", title="Source"),
                alt.Tooltip("ra:Q", title="RA (°)", format=".4f"),
                alt.Tooltip("dec:Q", title="Dec (°)", format=".4f"),
                alt.Tooltip("g_mag:Q", title="G magnitude", format=".2f"),
            ],
        )
        .properties(width="container", height=440)
    )
    mo.vstack([faintest, mo.md(f"Showing **{_shown.height:,}** of {glimpse.height:,} stars."), _chart])
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    Even in this small region, the richness of Gaia's data is striking — thousands of stars resolved in a fraction of a degree.

    ### Exploring the Data

    Before narrowing down to high-quality stars, it helps to take a broader look at the dataset, which carries 153 columns of measured and derived quantities for each star. Grouped logically:
    """)
    return


@app.cell(hide_code=True)
def _(mo):
    mo.accordion(
        {
            "Source identification & metadata": mo.md(r"""
    | Column | Meaning |
    | --- | --- |
    | `solution_id`, `designation`, `source_id` | Unique identifiers for each Gaia source and the data-processing version used. |
    | `random_index` | Random index number for internal or sampling purposes. |
    | `ref_epoch` | The reference time (in Julian years) when positions like RA/Dec and proper motion are defined. |
    | `duplicated_source` | True if multiple detections were merged. |
    | `has_*` | Indicate availability of detailed Gaia data products (XP spectra, epoch data, etc.). |
    | `in_andromeda_survey` | Flag if the source is part of the special Andromeda survey. |
    """),
            "Position in the sky: astrometry, coordinates and kinematics": mo.md(r"""
    | Column | Meaning |
    | --- | --- |
    | `ra`, `dec` | Right Ascension and Declination — the celestial coordinates. |
    | `ra_error`, `dec_error` | Uncertainty in those positions. |
    | `parallax` | Apparent annual shift due to Earth's orbit — used to measure **distance**. |
    | `parallax_error`, `parallax_over_error` | Uncertainty and reliability of parallax. |
    | `pmra`, `pmdec` | Proper motion — how fast the star moves across the sky in RA and Dec. |
    | `pm` | Total proper motion. |
    | `l`, `b` | Galactic longitude and latitude. |
    | `ecl_lon`, `ecl_lat` | Ecliptic coordinates. |
    | `radial_velocity`, `radial_velocity_error` | Line-of-sight velocity and its uncertainty. |
    | `vbroad`, `vbroad_error` | Projected rotational velocity. |
    """),
            "Astrometric quality": mo.md(r"""
    | Column | Meaning |
    | --- | --- |
    | `ra_dec_corr`, `ra_parallax_corr`, ... | Correlations between measurement errors. |
    | `astrometric_n_obs_al`, `astrometric_n_obs_ac` | Observations along/across Gaia's scan direction. |
    | `astrometric_gof_al`, `astrometric_chi2_al` | Goodness of fit — lower is a cleaner solution. |
    | `astrometric_excess_noise` | Extra jitter needed to fit — large values suggest unresolved binaries. |
    | `visibility_periods_used` | How many times the star was seen at different times. |
    | `ruwe` | *Renormalised Unit Weight Error* — values > 1.4 often mean a poor fit. |
    """),
            "Photometry": mo.md(r"""
    Gaia measures brightness in three bands: G (broad, 330–1050 nm), BP (blue, 330–680 nm) and RP (red, 630–1050 nm).

    | Column | Meaning |
    | --- | --- |
    | `phot_g_mean_flux`, `phot_bp_mean_flux`, `phot_rp_mean_flux` | Average detected light in each band. |
    | `phot_g_mean_mag`, `phot_bp_mean_mag`, `phot_rp_mean_mag` | Magnitudes. |
    | `phot_bp_rp_excess_factor` | BP+RP flux over G — high values suggest contamination. |
    | `bp_rp`, `bp_g`, `g_rp` | Colour indices, used to estimate temperature. |
    """),
            "Classification and astrophysical parameters": mo.md(r"""
    | Column | Meaning |
    | --- | --- |
    | `phot_variable_flag` | Brightness changes over time. |
    | `in_qso_candidates`, `in_galaxy_candidates`, `non_single_star` | Quasar, galaxy and multiple-star flags. |
    | `teff_gspphot` | Effective temperature (K). |
    | `logg_gspphot` | Surface gravity. |
    | `mh_gspphot` | Metallicity. |
    | `distance_gspphot` | Model-based distance (pc) — used for the plane maps below. |
    | `ag_gspphot`, `ebpminrp_gspphot` | Dust extinction and reddening. |
    """),
        }
    )
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ### Selecting High-Quality Stars

    RUWE measures how well each star's motion fits Gaia's astrometric model. RUWE ≈ 1 is a clean fit; above 1.4 suggests unresolved binaries, crowding or calibration issues. Keeping RUWE ≤ 1.4 preserves most of the catalog:

    ```sql
    SELECT COUNT(*) FROM gaiadr3.gaia_source;                  -- 1,811,709,771
    SELECT COUNT(*) FROM gaiadr3.gaia_source WHERE ruwe <= 1.4; -- 1,403,242,590
    ```

    ### Mapping the Whole Sky with HEALPix

    HEALPix divides the sky into equal-area pixels: N<sub>pix</sub> = 12 × 4<sup>order</sup>. Order 4 gives 3072 pixels, each about 3.7° across. Gaia aggregates on its own servers with `GAIA_HEALPIX_INDEX`, so instead of downloading a billion stars we download 3072 counts. Astropy then finds each pixel's centre, in equatorial and Galactic coordinates.
    """)
    return


@app.cell
async def _(adql, cached, local_only, pl):
    ORDER = 4

    def query_healpix():
        u = local_only("astropy.units")
        ICRS = local_only("astropy.coordinates").ICRS
        HEALPix = local_only("astropy_healpix").HEALPix

        counts = adql(f"""
            SELECT GAIA_HEALPIX_INDEX({ORDER}, source_id) AS healpix, COUNT(*) AS star_count
            FROM gaiadr3.gaia_source
            WHERE ruwe <= 1.4
            GROUP BY healpix
        """)
        centres = HEALPix(nside=2**ORDER, order="nested", frame=ICRS()).healpix_to_skycoord(
            counts["healpix"].cast(pl.Int64).to_numpy()
        )
        return counts.select(
            pl.col("healpix").cast(pl.Int32),
            pl.col("star_count").cast(pl.Int64),
            ra=pl.Series(centres.ra.deg),
            dec=pl.Series(centres.dec.deg),
            l=pl.Series(centres.galactic.l.wrap_at(180 * u.deg).deg),
            b=pl.Series(centres.galactic.b.deg),
        )

    healpix = await cached("healpix-density", query_healpix)
    healpix
    return (healpix,)


@app.cell
def _(COUNTS, alt):
    def sky_map(pixels, lon, lat, lon_title, lat_title):
        """An Equal Earth map of star counts per pixel; longitude increases to the left, as on the sky."""
        sphere = alt.Chart(alt.sphere()).mark_geoshape(fill=None, stroke="#888", strokeOpacity=0.5)
        graticule = alt.Chart(alt.graticule(step=[30, 30])).mark_geoshape(fill=None, stroke="#888", strokeOpacity=0.2)
        stars = (
            alt.Chart(pixels)
            .transform_calculate(flipped=f"-datum.{lon}")
            .mark_circle(size=42, opacity=1)
            .encode(
                longitude="flipped:Q",
                latitude=f"{lat}:Q",
                color=alt.Color("star_count:Q", title="Stars per pixel", scale=COUNTS, legend=alt.Legend(format="~s")),
                tooltip=[
                    alt.Tooltip("healpix:Q", title="HEALPix"),
                    alt.Tooltip(f"{lon}:Q", title=lon_title, format=".1f"),
                    alt.Tooltip(f"{lat}:Q", title=lat_title, format=".1f"),
                    alt.Tooltip("star_count:Q", title="Stars", format=","),
                ],
            )
        )
        return alt.layer(sphere, graticule, stars).project(type="equalEarth").properties(width="container", height=380)

    return (sky_map,)


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ## The Milky Way

    ### Stellar Density Across the Sky

    Plotted in equatorial coordinates (Right Ascension and Declination), the counts reveal a bright, thin band cutting across the sky — the Milky Way plane — where most stars are concentrated. Even at this moderate resolution, the bulge near the centre stands out.
    """)
    return


@app.cell
def _(healpix, sky_map):
    sky_map(healpix, "ra", "dec", "RA (°)", "Dec (°)")
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ### The Milky Way in Galactic Coordinates

    Equatorial coordinates suit observers on Earth but not the galaxy's geometry. Galactic coordinates put the Galactic Centre at (l = 0°, b = 0°) and run the plane along b = 0°. In this frame the structure is immediate: a narrow, dense strip, with the bulge in the middle.
    """)
    return


@app.cell
def _(healpix, sky_map):
    sky_map(healpix, "l", "b", "l (°)", "b (°)")
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    How thin is the disk? HEALPix pixels all cover the same area, so the share of pixels in a band is the share of sky it covers. Move the slider to see how much of the sky, and how many of the stars, lie within a band around the plane.
    """)
    return


@app.cell
def _(mo):
    band = mo.ui.slider(1, 45, value=10, label="Galactic latitude band, |b| ≤", show_value=True, full_width=True)
    band
    return (band,)


@app.cell
def _(band, healpix, mo, pl):
    _in_band = pl.col("b").abs() <= band.value
    _stars, _sky = healpix.select(
        pl.col("star_count").filter(_in_band).sum() / pl.col("star_count").sum(),
        _in_band.mean(),
    ).row(0)
    mo.hstack(
        [
            mo.stat(f"{_sky:.0%}", label=f"of the sky within ±{band.value}°"),
            mo.stat(f"{_stars:.0%}", label="of the stars"),
        ],
        justify="start",
        gap=2,
    )
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ### Mapping the Galactic Plane in Cartesian Bins

    To see the disk in physical space rather than angles, Galactic longitude, latitude and distance become Cartesian x and y around the Sun, binned on Gaia's server with `FLOOR()`. Each distance limit keeps about 5000 bins, so a smaller limit gives finer bins: 141 pc across at 5000 pc.
    """)
    return


@app.cell
async def _(adql, cached, math, pl):
    def query_plane(limit):
        bin_size = math.floor(limit / math.sqrt(5000) * 2)
        counts = adql(f"""
            SELECT
              FLOOR(distance_gspphot * COS(RADIANS(b)) * COS(RADIANS(l)) / {bin_size}) * {bin_size} AS x_bin,
              FLOOR(distance_gspphot * COS(RADIANS(b)) * SIN(RADIANS(l)) / {bin_size}) * {bin_size} AS y_bin,
              COUNT(*) AS star_count
            FROM gaiadr3.gaia_source
            WHERE ruwe <= 1.4
              AND distance_gspphot > 0
              AND distance_gspphot <= {limit}
            GROUP BY x_bin, y_bin
        """)
        return counts.select(
            distance_limit=pl.lit(limit, pl.Int32),
            bin_size=pl.lit(bin_size, pl.Int32),
            x=pl.col("x_bin").cast(pl.Float64),
            y=pl.col("y_bin").cast(pl.Float64),
            star_count=pl.col("star_count").cast(pl.Int64),
        )

    plane_density = await cached("plane-density", lambda: pl.concat([query_plane(5000), query_plane(3000)]))
    plane_density
    return (plane_density,)


@app.cell
def _(COUNTS, alt, pl):
    def plane_map(plane_density, limit):
        """Star counts per bin on the Galactic plane, the Sun at the origin. A fixed square, as x and y are
        both parsecs; on a narrow screen it scrolls sideways in its own box."""
        bins = plane_density.filter(pl.col("distance_limit") == limit).with_columns(
            x_end=pl.col("x") + pl.col("bin_size"), y_end=pl.col("y") + pl.col("bin_size")
        )
        reach = float(bins.select(pl.max_horizontal(-pl.col("x"), "x_end", -pl.col("y"), "y_end").max()).item())
        scale = alt.Scale(domain=[-reach, reach], nice=False)
        return (
            alt.Chart(bins)
            .mark_rect()
            .encode(
                x=alt.X("x:Q", title="Galactic X (pc), towards the Galactic Centre →", scale=scale),
                x2="x_end:Q",
                y=alt.Y("y:Q", title="↑ Galactic Y (pc), direction of rotation", scale=scale),
                y2="y_end:Q",
                color=alt.Color(
                    "star_count:Q",
                    title="Stars per bin",
                    scale=COUNTS,
                    legend=alt.Legend(orient="bottom", direction="horizontal", gradientLength=240, format="~s"),
                ),
                tooltip=[
                    alt.Tooltip("x:Q", title="X (pc)"),
                    alt.Tooltip("y:Q", title="Y (pc)"),
                    alt.Tooltip("star_count:Q", title="Stars", format=","),
                ],
            )
            .properties(width=480, height=480)
        )

    return (plane_map,)


@app.cell
def _(plane_density, plane_map):
    plane_map(plane_density, 5000)
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ### Refining the Distance Limit

    At 5000 pc, nearby structure is compressed near the centre. Limiting the data to 3000 pc, with the same number of bins, makes each bin 84 pc across and sharpens the local disk. The Sun sits at (0, 0).

    This chart is selectable: drag a box over part of it, and the cell below counts the stars inside.
    """)
    return


@app.cell
def _(mo, plane_density, plane_map):
    near = mo.ui.altair_chart(plane_map(plane_density, 3000), chart_selection="interval", legend_selection=False)
    near
    return (near,)


@app.cell
def _(mo, near):
    _picked = near.value
    if _picked.is_empty():
        _out = mo.md("_Drag a box on the chart above to count the stars in it._")
    else:
        _out = mo.hstack(
            [
                mo.stat(f"{int(_picked['star_count'].sum()):,}", label="stars selected"),
                mo.stat(f"{_picked.height:,}", label="bins"),
                mo.stat(f"{int(_picked['star_count'].max()):,}", label="in the densest bin"),
            ],
            justify="start",
            gap=2,
        )
    _out
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ## Conclusion

    The Gaia DR3 dataset can be explored systematically to map the structure of the Milky Way. HEALPix sky binning, Galactic coordinate transformations and Cartesian projections turn raw astrometry into density maps that show the Galaxy's thin stellar disk, central bulge and large-scale symmetry, and changing distance limits or bin sizes brings out different structure.

    Additional Gaia parameters, such as colour, magnitude or proper motion, could extend this to the Galaxy's stellar populations and kinematics.

    ## Acknowledgements

    This work has made use of data from the European Space Agency (ESA) mission [Gaia](https://www.cosmos.esa.int/gaia), processed by the Gaia Data Processing and Analysis Consortium ([DPAC](https://www.cosmos.esa.int/web/gaia/dpac/consortium)). Funding for the DPAC has been provided by national institutions, in particular the institutions participating in the Gaia Multilateral Agreement.
    """)
    return


if __name__ == "__main__":
    app.run()

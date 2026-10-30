# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "astropy",
#     "astropy-healpix",
#     "astroquery",
#     "marimo",
#     "polars",
# ]
# ///
"""Mapping the Milky Way: the Gaia DR3 queries behind the post, and the data its charts read."""

import marimo

__generated_with = "0.25.2"
app = marimo.App(width="medium")


@app.cell
def _():
    import math

    import marimo as mo
    import polars as pl

    POST = "mapping-the-milky-way"
    # Gaia queries are slow, so their results are cached here; set REFRESH to query the archive again.
    CACHE = mo.notebook_dir() / "data" / POST
    REFRESH = False
    OUT = mo.notebook_dir().parent / "owlmanac-web" / "src" / "content" / "posts" / POST / "data"
    OUT.mkdir(parents=True, exist_ok=True)
    return CACHE, OUT, REFRESH, math, mo, pl


@app.cell
def _(CACHE, REFRESH, pl):
    def cached(name, fetch):
        """Reads a cached Gaia result, or calls `fetch` (returning an astropy Table) and caches it."""
        path = CACHE / f"{name}.parquet"
        if REFRESH or not path.exists():
            path.parent.mkdir(parents=True, exist_ok=True)
            pl.from_pandas(fetch().to_pandas()).write_parquet(path)
        return pl.read_parquet(path)

    def adql(query):
        def fetch():
            from astroquery.gaia import Gaia

            return Gaia.launch_job_async(query).get_results()

        return fetch

    return adql, cached


@app.cell
def _(mo):
    mo.md("""
    ## First glimpse: a 0.2° patch towards Sagittarius A*
    """)
    return


@app.cell
def _(OUT, cached, pl):
    GLIMPSE = {"ra": 266.4, "dec": -29.0, "width": 0.2}

    def query_glimpse():
        import astropy.units as u
        from astropy.coordinates import SkyCoord
        from astroquery.gaia import Gaia

        Gaia.MAIN_GAIA_TABLE = "gaiadr3.gaia_source"
        Gaia.ROW_LIMIT = 5000
        side = u.Quantity(GLIMPSE["width"], u.deg)
        coord = SkyCoord(ra=GLIMPSE["ra"], dec=GLIMPSE["dec"], unit=(u.deg, u.deg), frame="icrs")
        return Gaia.query_object_async(coordinate=coord, width=side, height=side)

    glimpse = (
        cached("glimpse_center", query_glimpse)
        .select(pl.col("source_id").cast(pl.String), "ra", "dec", g_mag="phot_g_mean_mag")
        .drop_nulls()
    )
    glimpse.write_parquet(OUT / "glimpse.parquet")
    glimpse
    return


@app.cell
def _(mo):
    mo.md("""
    ## Star counts per HEALPix pixel (order 4, RUWE ≤ 1.4)

    The pixel centres are computed here, in equatorial (RA, Dec) and Galactic (l, b) coordinates, so the browser only plots.
    """)
    return


@app.cell
def _(OUT, adql, cached, pl):
    import astropy.units as u
    import numpy as np
    from astropy.coordinates import ICRS
    from astropy_healpix import HEALPix

    ORDER = 4
    counts = cached(
        "healpix_density",
        adql(f"""
            SELECT GAIA_HEALPIX_INDEX({ORDER}, source_id) AS healpix_{ORDER}, COUNT(*) AS star_count
            FROM gaiadr3.gaia_source
            WHERE ruwe <= 1.4
            GROUP BY healpix_{ORDER}
        """),
    )
    centres = HEALPix(nside=2**ORDER, order="nested", frame=ICRS()).healpix_to_skycoord(
        counts[f"healpix_{ORDER}"].to_numpy().astype(np.int64)
    )
    healpix = counts.select(
        healpix=pl.col(f"healpix_{ORDER}").cast(pl.Int32),
        star_count=pl.col("star_count").cast(pl.Int64),
        ra=pl.Series(centres.ra.deg),
        dec=pl.Series(centres.dec.deg),
        l=pl.Series(centres.galactic.l.wrap_at(180 * u.deg).deg),
        b=pl.Series(centres.galactic.b.deg),
    )
    healpix.write_parquet(OUT / "healpix-density.parquet")
    healpix
    return


@app.cell
def _(mo):
    mo.md("""
    ## Star counts on the Galactic plane, binned in x and y around the Sun

    Each distance limit keeps about 5000 bins, so a smaller limit gives finer bins.
    """)
    return


@app.cell
def _(OUT, adql, cached, math, pl):
    def plane(limit):
        bin_size = math.floor(limit / math.sqrt(5000) * 2)
        counts = cached(
            f"galactic_plane_density_{limit}",
            adql(f"""
                SELECT
                  FLOOR(distance_gspphot * COS(RADIANS(b)) * COS(RADIANS(l)) / {bin_size}) * {bin_size} AS x_bin,
                  FLOOR(distance_gspphot * COS(RADIANS(b)) * SIN(RADIANS(l)) / {bin_size}) * {bin_size} AS y_bin,
                  COUNT(*) AS star_count
                FROM gaiadr3.gaia_source
                WHERE ruwe <= 1.4
                  AND distance_gspphot > 0
                  AND distance_gspphot <= {limit}
                GROUP BY x_bin, y_bin
            """),
        )
        return counts.select(
            distance_limit=pl.lit(limit, pl.Int32),
            bin_size=pl.lit(bin_size, pl.Int32),
            x=pl.col("x_bin").cast(pl.Float64),
            y=pl.col("y_bin").cast(pl.Float64),
            star_count=pl.col("star_count").cast(pl.Int64),
        )

    plane_density = pl.concat([plane(5000), plane(3000)])
    plane_density.write_parquet(OUT / "plane-density.parquet")
    plane_density
    return


if __name__ == "__main__":
    app.run()

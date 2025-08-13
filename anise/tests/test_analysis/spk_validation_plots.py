from glob import glob
from os import environ
from os.path import abspath, basename, dirname, join

import pandas as pd
import plotly.express as px


def is_on_github_actions():
    return ("CI" in environ and environ["CI"] and "GITHUB_RUN_ID" in environ)


if __name__ == '__main__':

    target_folder = join(abspath(dirname(__file__)), '..', '..', '..', 'target')

    plotted_anything = False

    for filename in glob(f"{target_folder}/spk-type*.parquet"):
        print(f"reading {filename}")
        # Load the parquet file
        df = pd.read_parquet(filename)
        df = df[df["source frame"]=="-74"]

        name = basename(filename)

        for kind, columns in [("Position", ["X", "Y", "Z"]),
                              ("Velocity", ["VX", "VY", "VZ"])]:

            print(f"== {kind} {name} ==")

            subset = df.loc[df.component.isin(columns)]
            subset["error"] = (subset["ANISE value"] - subset["SPICE value"])/subset["SPICE value"]

            print(subset.describe())

            plt = px.scatter(subset,
                             x='ET Epoch (s)',
                             y="error")

            plt.write_html(
                f"{target_folder}/validation-plot-{kind}-{name}.html")
            if not is_on_github_actions():
                plt.show()
            plotted_anything = True

        # Plot all components together
        plt = px.scatter(df,
                         x='ET Epoch (s)',
                         y='Absolute difference',
                         color='component',
                         title=f"Validation of {name} (overall)")
        plt.write_html(f"{target_folder}/validation-plot-{name}.html")
        if not is_on_github_actions():
            plt.show()
        plotted_anything = True
    assert plotted_anything, "did not plot anything"

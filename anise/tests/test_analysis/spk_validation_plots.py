from glob import glob
from os import environ
from os.path import abspath, basename, dirname, join

import pandas as pd
import plotly.express as px


def is_on_github_actions():
  if "CI" not in environ or not environ["CI"] or "GITHUB_RUN_ID" not in environ:
    return False
  else:
    return True

if __name__ == '__main__':

    target_folder = join(abspath(dirname(__file__)), '..', '..', '..', 'target')

    plotted_anything = False

    for filename in glob(f"{target_folder}/spk-type*.parquet"):

        # Load the parquet file
        df = pd.read_parquet(filename)

        name = basename(filename)

        for kind, columns in [("Position", ["X", "Y", "Z"]),
                              ("Velocity", ["VX", "VY", "VZ"])]:

            print(f"== {kind} {name} ==")

            subset = df.loc[df.component.isin(columns)]

            print(subset.describe())

            plt = px.scatter(subset,
                             x='ET Epoch (s)',
                             y=f'Absolute difference',
                             color='source frame',
                             title=f"Validation of {name} for {kind}")

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

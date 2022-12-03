import pandas as pd
import plotly.express as px
from os.path import abspath, join, dirname

if __name__ == '__main__':

    target_folder = join(abspath(dirname(__file__)), '..', 'target')

    for name, filename in [("validation", "validation-test-results"),
                           ("outliers", "validation-outliers")]:

        # Load the parquet file
        df = pd.read_parquet(f"{target_folder}/{filename}.parquet")

        if name == 'validation':
            y = 'relative error'
        else:
            y = 'absolute error'

        for src in ['de438s', 'de440']:
            for kind, columns in [("Position", ["X", "Y", "Z"]),
                                  ("Velocity", ["VX", "VY", "VZ"])]:

                print(f"== {kind} {name} {src} ==")

                subset = df.loc[df['DE file'] == src].loc[df.component.isin(
                    columns)]

                print(subset.describe())

                plt = px.scatter(subset,
                                 x='File delta T (s)',
                                 y=y,
                                 color='source frame')

                plt.write_html(
                    f"{target_folder}/{src}-{name}-{kind}-validation.html")
                plt.show()

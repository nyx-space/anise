import pandas as pd
import plotly.express as px
from os.path import abspath, join, dirname

if __name__ == '__main__':

    target_folder = join(abspath(dirname(__file__)), '..', 'target')

    for name, filename in [("validation", "type13-validation-test-results"),
                           ("outliers", "type13-validation-outliers")]:

        # Load the parquet file
        df = pd.read_parquet(f"{target_folder}/{filename}.parquet")

        if name == 'validation':
            y = 'relative error'
        else:
            y = 'absolute error'

        for kind, columns in [("Position", ["X", "Y", "Z"]),
                                ("Velocity", ["VX", "VY", "VZ"])]:

            print(f"== {kind} {name} ==")

            subset = df.loc[df.component.isin(columns)]

            print(subset.describe())

            plt = px.scatter(subset,
                                x='File delta T (s)',
                                y=y,
                                color='source frame')

            plt.write_html(
                f"{target_folder}/{name}-{kind}-validation.html")
            plt.show()

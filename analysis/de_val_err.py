import pandas as pd
import plotly.express as px

df = pd.read_parquet("../target/validation-test-results.parquet")

print(df.describe())

px.scatter(df, x='source frame', y=['relative error', 'absolute error'], color='component', text='destination frame').write_html("../target/validation-mean-min-max.html")

import pandas as pd
import plotly.express as px

df = pd.read_parquet("../target/validation-test-results.parquet")

print(df.loc[df['DE file'] == 'de438s'].describe())

plt = px.scatter(df.loc[df['DE file'] == 'de438s'], x='File delta T (s)', y='relative error (km)', color='source frame')

plt.write_html("../target/de438s-validation.html")
plt.show()


print(df.loc[df['DE file'] == 'de440'].describe())
plt = px.scatter(df.loc[df['DE file'] == 'de440'], x='File delta T (s)', y='relative error (km)', color='source frame')

plt.write_html("../target/de440-validation.html")
plt.show()
from langchain_community.tools import DuckDuckGoSearchResults

search = DuckDuckGoSearchResults(max_results=3)

result = search.invoke("What is the capital of France?")
print(result)

secrets:add {
	url = "https://httpbin.org"
}

secrets:add("dev") {
	url = "http://localhost:9090"
}

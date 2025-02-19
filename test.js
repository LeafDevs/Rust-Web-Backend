const authRequest = async () => {
    const response = await fetch("http://127.0.0.1:8080/api/v1/auth", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        authtype: "login",
        email: "example@example.com",
        password: "yourpassword",
        firstname: "John",
        lastname: "Doe"
      })
    });
  
    const data = await response.text();
    console.log(data);
  };
  
  authRequest();
  
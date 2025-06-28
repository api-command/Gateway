from locust import HttpUser, task, between, TaskSet, tag
import random
import uuid

class GatewayUser(HttpUser):
    host = "http://localhost:8000"  # Update with your gateway URL
    wait_time = between(1, 5)  # Random wait between 1-5 seconds

    def on_start(self):
        """Authenticate user and get JWT token"""
        self.token = self.login()
        self.headers = {"Authorization": f"Bearer {self.token}"}
        
    def login(self):
        response = self.client.post("/auth/login", json={
            "username": "testuser",
            "password": "testpass"
        })
        return response.json().get("access_token")

    @tag('general')
    @task(5)
    def get_public_content(self):
        """Test unauthenticated content delivery"""
        self.client.get("/public/content")

    @tag('auth')
    @task(3)
    def get_private_content(self):
        """Test authenticated endpoint"""
        self.client.get("/private/data", headers=self.headers)

    @tag('auth', 'write')
    @task(2)
    def post_data(self):
        """Test write operation with payload"""
        payload = {
            "id": str(uuid.uuid4()),
            "value": random.randint(1, 1000)
        }
        self.client.post("/data", json=payload, headers=self.headers)

    @tag('search')
    @task(1)
    def search_operation(self):
        """Test search with query parameters"""
        query = random.choice(["book", "movie", "music", "game"])
        self.client.get(f"/search?q={query}", headers=self.headers)

    @tag('cache')
    @task(4)
    def get_cached_content(self):
        """Test cache hit performance"""
        self.client.get("/cached/content")

    @tag('ratelimit')
    @task(1)
    def test_rate_limits(self):
        """Intentionally trigger rate limits"""
        for _ in range(20):
            self.client.get("/api/limited", headers=self.headers)

    @tag('error')
    @task(1)
    def test_error_conditions(self):
        """Test error scenarios"""
        # Invalid auth
        self.client.get("/private/data", headers={"Authorization": "Bearer invalid"})
        # Invalid endpoint
        self.client.get("/invalid/endpoint")
        # Large payload
        self.client.post("/data", json={"data": "x"*10000}, headers=self.headers)

from locust import SequentialTaskSet

class StressTest(SequentialTaskSet):
    """Special scenario for maximum stress testing"""
    @task
    def flood_requests(self):
        endpoints = [
            "/public/content",
            "/private/data",
            "/search?q=test",
            "/cached/content"
        ]
        for _ in range(50):
            url = random.choice(endpoints)
            self.client.get(url)

class MixedWorkloadUser(HttpUser):
    wait_time = between(0.1, 0.5)
    tasks = [GatewayUser, StressTest]
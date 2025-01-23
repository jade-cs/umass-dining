import asyncio
import aiohttp
import random
import urllib.parse
import time
from statistics import mean, stdev


async def fetch_list(base_url, session):
    """Fetch the list of dining halls from the base endpoint."""
    try:
        async with session.get(f"{base_url}/") as response:
            response.raise_for_status()
            return await response.json()
    except Exception as e:
        print(f"Error fetching list: {e}")
        return []


async def fetch_info(base_url, venue_name, session, latencies):
    """Fetch information for a specific venue and record latency."""
    try:
        start_time = time.perf_counter()
        endpoint = f"{base_url}/{urllib.parse.quote(venue_name)}"
        async with session.get(endpoint) as response:
            await response.text()  # Optionally process the response
        elapsed_time = time.perf_counter() - start_time
        latencies.append(elapsed_time)
    except Exception as e:
        print(f"Error fetching venue info for {venue_name}: {e}")


async def make_requests(base_url, venues, latencies, duration_seconds):
    """Make random requests to venue endpoints for a specific duration."""
    async with aiohttp.ClientSession() as session:
        end_time = time.time() + duration_seconds
        while time.time() < end_time:
            venue_name = random.choice(venues)
            await fetch_info(base_url, venue_name, session, latencies)


async def main():
    base_url = "http://localhost:8000"  # Replace with your server's base URL
    duration_seconds = 10 * 60  # 10 minutes
    latencies = []

    async with aiohttp.ClientSession() as session:
        venues = await fetch_list(base_url, session)

    if not venues:
        print("No venues found. Exiting.")
        return

    print(f"Fetched {len(venues)} venues. Starting requests...")
    await make_requests(base_url, venues, latencies, duration_seconds)

    # Calculate statistics
    if latencies:
        mean_latency = mean(latencies)
        stdev_latency = stdev(latencies) if len(latencies) > 1 else 0
        min_latency = min(latencies)
        max_latency = max(latencies)

        print("\nLatency Statistics:")
        print(f"Mean Latency: {mean_latency:.4f} seconds")
        print(f"Standard Deviation: {stdev_latency:.4f} seconds")
        print(f"Minimum Latency: {min_latency:.4f} seconds")
        print(f"Maximum Latency: {max_latency:.4f} seconds")
    else:
        print("No successful requests recorded.")


if __name__ == "__main__":
    asyncio.run(main())


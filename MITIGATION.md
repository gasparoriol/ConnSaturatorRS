## Resiliency and Mitigation Strategies

ConnSaturatorRS is designed to test the limits of a web service and evaluate its resiliency under heavy load. In this case it's an application created to test the implementation of a rate limiter in a Spring Boot application.


### 1. Rate Limiting
The most effective way to prevent a connection saturator is to limit how many requests can be sent to the target server from the same IP address. This can be achieved by implementing a rate limiter in the application. 

**Spring Boot Rate Limiter Example (with Bucket4j):**
In a Spring Boot application, you can implement a rate limiter using Bucket4j and Token Bucket algorithm. If a client exceeds the rate limit, the request is rejected with a 429 (Too Many Requests) response.

A potential solution to this problem is this:


**Configuration Example:**

In the config file we can define two rate limiters, one for the application and one for the login.
```java
@Configuration
public class RateLimiterConfig {

  private static final int MAXREQUESTS = 50;
  private static final int MAXREQUESTSLOGIN = 5;
  private static final int MAXDURATION = 1;

  public Bandwidth getRateLimit() {    
    return Bandwidth.classic(MAXREQUESTS, Refill.intervally(MAXREQUESTS, Duration.ofMinutes(MAXDURATION)));
  }

    public Bandwidth getRateLimitForLogin() {    
    return Bandwidth.classic(MAXREQUESTSLOGIN, Refill.intervally(MAXREQUESTSLOGIN, Duration.ofMinutes(MAXDURATION)));
  }
}
```

**Rate Limiter Service Example:**

In the rate limiter service we can define a map of buckets, one for each IP address and path.

```java
@Service
public class RateLimiterService {

  @Autowired
  private RateLimiterConfig rateLimiterConfig;

  private final Map<String, Bucket> buckets = new ConcurrentHashMap<>();

  public Bucket resolveBucket(String ip, String path) {
    String key = ip + (path.contains("/login") ? "login" : "general");

    return buckets.computeIfAbsent(key, k -> {
      if (key.contains("login")) {
        return Bucket.builder()
            .addLimit(rateLimiterConfig.getRateLimitForLogin())
            .build();
      } else {
        return Bucket.builder()
            .addLimit(rateLimiterConfig.getRateLimit())
            .build();
      }
    });
  }
}
```

**Rate Limiter Filter Example:**

In the rate limiter filter we can define a filter that will be executed for each request.
We initialize the ip variable with the *X-Forwarded-For* for the possibility of using a proxy.
If the *X-Forwarded-For* is not present we use the remote address.

ConsumptionProbe is used to try to consume a token from the bucket to determine the waiting time it will take to refill the bucket.
If the token is consumed, the request is allowed to proceed.
If the token is not consumed we can redirect user to a static page or return or a JSON response with a 429 status code and a message.

```java
@Component
public class RateLimiterFilter implements OncePerRequestFilter {

  @Autowired
  private RateLimiterService rateLimiterService;

  private static final long MAX_WAIT_TIME_MS = 3000;

  @Override
  protected void doFilterInternal(@NonNull HttpServletRequest request, 
                                @NonNull HttpServletResponse response, 
                                @NonNull FilterChain filterChain) throws ServletException, IOException {
                                
    String ip = request.getHeader("X-Forwarded-For");
    if (ip == null || ip.isEmpty()) {
      ip = request.getRemoteAddr();
    } else {
      ip = ip.split(",")[0].trim();
    }

    String path = request.getRequestURI();

    Bucket bucket = rateLimiterService.resolveBucket(ip, path);

    ConsumptionProbe probe = bucket.tryConsumeAndReturnRemaining(1);
    
    if (probe.getConsumed() == 1) {
      filterChain.doFilter(request, response);
    } else {
      
      response.setStatus(HttpServletResponse.SC_TOO_MANY_REQUESTS); 
      response.setContentType("application/json");
      String degradedResponse = "{" +
            "\"status\": \"degraded\"," +
            "\"message\": \"Our systems are under heavy load. Serving simplified content.\"," +
            "\"retry_after_seconds\": " + TimeUnit.NANOSECONDS.toSeconds(probe.getNanosToWaitForRefill()) +
            "}";
            
      response.getWriter().write(degradedResponse);
      response.getWriter().flush();
    }   
  }
}
```

**Service Degradation vs. Thread Blocking:**

Service degradation and Thread blocking are two different strategies to prevent a failure.

Service degradation, it's considered safe, inmediatly releases the thread and allows it to be used for other requests and return a lightweight response.

Thread blocking, it's considered unsafe, dangerous. Keeping a thread alive while waiting for rate-limit refill, consumes server memory and connection pools.

If we use *Thread.sleep()* to mitigate the saturation it is the most expensive way to do it because we could end up all the threads of the pool to sleep and not be able to handle any new requests.


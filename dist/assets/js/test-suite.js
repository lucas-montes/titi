// Test Suite Dashboard JavaScript

// Timeline Chart
const timelineOptions = {
    series: [{
        name: 'Tests Executed',
        data: generateTimeSeriesData(60, { min: 20, max: 50 })
    }, {
        name: 'Tests Passed',
        data: generateTimeSeriesData(60, { min: 18, max: 48 })
    }, {
        name: 'Tests Failed',
        data: generateTimeSeriesData(60, { min: 0, max: 5 })
    }],
    chart: {
        type: 'area',
        height: 350,
        stacked: false,
        zoom: {
            enabled: true,
            type: 'x',
            autoScaleYaxis: true
        },
        toolbar: {
            show: true,
            autoSelected: 'zoom'
        },
        animations: {
            enabled: true,
            easing: 'linear',
            dynamicAnimation: {
                speed: 1000
            }
        }
    },
    colors: ['#4680ff', '#2ca87f', '#dc2626'],
    dataLabels: {
        enabled: false
    },
    stroke: {
        curve: 'smooth',
        width: 2
    },
    fill: {
        type: 'gradient',
        gradient: {
            opacityFrom: 0.6,
            opacityTo: 0.1,
        }
    },
    xaxis: {
        type: 'datetime',
        labels: {
            datetimeUTC: false
        }
    },
    yaxis: {
        title: {
            text: 'Number of Tests'
        },
        labels: {
            formatter: function(val) {
                return val.toFixed(0);
            }
        }
    },
    legend: {
        position: 'top',
        horizontalAlign: 'left'
    },
    tooltip: {
        shared: true,
        x: {
            format: 'HH:mm:ss'
        }
    }
};

const timelineChart = new ApexCharts(document.querySelector("#timeline-chart"), timelineOptions);
timelineChart.render();

// Distribution Chart (Donut)
const distributionOptions = {
    series: [250, 180, 515, 300],
    chart: {
        type: 'donut',
        height: 350
    },
    labels: ['Security', 'Compliance', 'Correctness', 'Performance'],
    colors: ['#dc2626', '#3b82f6', '#4680ff', '#f59e0b'],
    legend: {
        position: 'bottom'
    },
    dataLabels: {
        enabled: true,
        formatter: function(val, opts) {
            return opts.w.config.series[opts.seriesIndex];
        }
    },
    plotOptions: {
        pie: {
            donut: {
                size: '70%',
                labels: {
                    show: true,
                    name: {
                        show: true
                    },
                    value: {
                        show: true,
                        fontSize: '22px',
                        fontWeight: 600,
                        formatter: function(val) {
                            return val + ' tests';
                        }
                    },
                    total: {
                        show: true,
                        label: 'Total Tests',
                        formatter: function(w) {
                            return w.globals.seriesTotals.reduce((a, b) => a + b, 0);
                        }
                    }
                }
            }
        }
    }
};

const distributionChart = new ApexCharts(document.querySelector("#distribution-chart"), distributionOptions);
distributionChart.render();

// Helper function to generate time series data
function generateTimeSeriesData(count, yrange) {
    const series = [];
    const now = new Date().getTime();

    for (let i = count; i >= 0; i--) {
        const x = now - (i * 60000); // 1-minute intervals
        const y = Math.floor(Math.random() * (yrange.max - yrange.min + 1)) + yrange.min;
        series.push({
            x: x,
            y: y
        });
    }

    return series;
}

// Load Test Runs
function loadTestRuns() {
    const tbody = document.getElementById('test-runs-table');

    // Sample data - replace with actual API call
    const testRuns = [
        {
            suite: 'User API Tests',
            phase: 'Security',
            status: 'completed',
            duration: '2m 34s',
            total: 250,
            passed: 236,
            failed: 14,
            successRate: 94.4,
            started: '10 minutes ago'
        },
        {
            suite: 'Product API Tests',
            phase: 'Correctness',
            status: 'running',
            duration: '1m 12s',
            total: 180,
            passed: 156,
            failed: 2,
            successRate: 98.7,
            started: '1 minute ago'
        },
        {
            suite: 'Order API Tests',
            phase: 'Compliance',
            status: 'completed',
            duration: '3m 45s',
            total: 180,
            passed: 178,
            failed: 2,
            successRate: 98.9,
            started: '1 hour ago'
        },
        {
            suite: 'Payment API Tests',
            phase: 'Security',
            status: 'failed',
            duration: '45s',
            total: 120,
            passed: 89,
            failed: 31,
            successRate: 74.2,
            started: '3 hours ago'
        }
    ];

    tbody.innerHTML = testRuns.map(run => `
        <tr>
            <td><strong>${run.suite}</strong></td>
            <td>
                <span class="badge bg-${getPhaseColor(run.phase)}">${run.phase}</span>
            </td>
            <td>
                <span class="badge bg-${getStatusColor(run.status)}">${run.status}</span>
            </td>
            <td>${run.duration}</td>
            <td>
                <span class="text-success">${run.passed}</span> /
                <span class="text-danger">${run.failed}</span> /
                <span class="text-muted">${run.total}</span>
            </td>
            <td>
                <div class="d-flex align-items-center">
                    <div class="progress flex-grow-1 me-2" style="height: 8px; min-width: 80px;">
                        <div class="progress-bar bg-${run.successRate > 95 ? 'success' : run.successRate > 80 ? 'warning' : 'danger'}"
                             style="width: ${run.successRate}%"></div>
                    </div>
                    <span class="small">${run.successRate}%</span>
                </div>
            </td>
            <td><small class="text-muted">${run.started}</small></td>
            <td>
                <button class="btn btn-sm btn-outline-primary" onclick="viewTestDetails('${run.suite}')">
                    <i class="bi bi-eye"></i>
                </button>
                <button class="btn btn-sm btn-outline-secondary" onclick="rerunTest('${run.suite}')">
                    <i class="bi bi-arrow-clockwise"></i>
                </button>
            </td>
        </tr>
    `).join('');
}

function getPhaseColor(phase) {
    const colors = {
        'Security': 'danger',
        'Compliance': 'info',
        'Correctness': 'primary',
        'Performance': 'warning'
    };
    return colors[phase] || 'secondary';
}

function getStatusColor(status) {
    const colors = {
        'completed': 'success',
        'running': 'primary',
        'failed': 'danger',
        'queued': 'secondary'
    };
    return colors[status] || 'secondary';
}

function viewTestDetails(suite) {
    console.log('Viewing details for:', suite);
    // Implement modal or navigation
}

function rerunTest(suite) {
    console.log('Re-running test:', suite);
    // Implement test re-run
}

// Auto-refresh data every 5 seconds
let refreshInterval = setInterval(() => {
    loadTestRuns();
    updateMetrics();
}, 5000);

function updateMetrics() {
    // Simulate live updates
    const totalTests = document.getElementById('total-tests');
    const passedTests = document.getElementById('passed-tests');
    const failedTests = document.getElementById('failed-tests');
    const successRate = document.getElementById('success-rate');

    if (totalTests) {
        const current = parseInt(totalTests.textContent.replace(',', ''));
        totalTests.textContent = (current + Math.floor(Math.random() * 5)).toLocaleString();
    }
}

// Modal toggles
document.querySelectorAll('input[name="specSource"]').forEach(radio => {
    radio.addEventListener('change', function() {
        const urlInput = document.getElementById('urlInput');
        const fileInput = document.getElementById('fileInput');

        if (this.id === 'specUrl') {
            urlInput.style.display = 'block';
            fileInput.style.display = 'none';
        } else {
            urlInput.style.display = 'none';
            fileInput.style.display = 'block';
        }
    });
});

// Initial load
document.addEventListener('DOMContentLoaded', function() {
    loadTestRuns();
});

// Cleanup on page unload
window.addEventListener('beforeunload', function() {
    if (refreshInterval) {
        clearInterval(refreshInterval);
    }
});

var data_url = "/donation_data";
var data_update_url = "/donation_data/update";

$(function() {
	$.get(data_url).done(function(fullData) {
        initWithData(fullData);

        //now that we have our charts, start refreshing the data every 5 minutes
        var mostRecentUpdate = new Date();

        var performUpdate = function() {
            $.get(data_update_url, {'since':mostRecentUpdate.toISOString()}).done(function(partialData) {
                fullData = fullData.concat(partialData);
                initWithData(fullData);

                mostRecentUpdate = new Date();

                //do another update in 5 minutes
                setTimeout(performUpdate, getRefreshTimeout());
            });
        }
        setTimeout(performUpdate, getRefreshTimeout());
    });

    //auto refresh the whole page after 2 hours
    setTimeout(function() {
        window.location.reload(true);
    }, 2 * 60 * 60 * 1000);
});

var getRefreshTimeout = function() {
    //5 minutes, plus a random value between 0 and 30 seconds
    return 5 * 60 * 1000 + 30 *1000 * Math.random();
}


var initWithData = function(donationData) {
	var totals = donationData.map(function(item) {
		return item.total;
	});
	var timestamps = donationData.map(function(item) {
		return Date.parse(item.timestamp);
	});

	//create a series for dollars per minute
	var hourlyTimeStamps
	var dollarsPerHour = [];
	var dollarsPerDonation = [];
	for (var i = 0; i < totals.length - 1; i++) {
		var deltaTotal = totals[i + 1] - totals[i];
		var hoursBetweenSamples = (timestamps[i + 1] - timestamps[i]) / (1000 * 60 * 60);
		dollarsPerHour.push(deltaTotal / (hoursBetweenSamples > 0 ? hoursBetweenSamples : 1));
	}
	ironOutTopPercent(dollarsPerHour, 0.025);
	exponentialSmooth(dollarsPerHour, 0.4);

	//data series for dollars per donation
	var dollarsPerDonation = [];
	for (var i = 0; i < totals.length - 1; i++) {
		var deltaTotal = totals[i + 1] - totals[i];
		var deltaCount = donationData[i + 1].count - donationData[i].count;
		dollarsPerDonation.push(deltaTotal / (deltaCount > 0 ? deltaCount : 1));
	}
	ironOutTopPercent(dollarsPerDonation, 0.025);
	exponentialSmooth(dollarsPerDonation, 0.4);

	//create our various charts
	var totals_chart = createTotalsChart(timestamps, totals);

	//remove the first timestamp since we'll no longer be using it
	timestamps.splice(0,1);
	var rate_chart = createRateChart(timestamps, dollarsPerHour);
	var per_donation_chart = createPerDonationChart(timestamps, dollarsPerDonation);
}

var createTotalsChart = function(timestamps, totals) {
	Highcharts.setOptions({
        global: {
            useUTC: false
        }
    });

    return Highcharts.chart('total-container', {
        chart: {
            type: 'spline',
            animation: Highcharts.svg,
        },
        title: {
            text: 'Unofficial AGDQ 2017 Donation Chart',
        },
        subtitle: {
            text: "No refresh necessary: Data auto-updates every 5 minutes",
        },
        xAxis: {
            type: 'datetime',
            labels: {
                format: '{value:%a %l%P}',
            },
        },
        yAxis: { // Primary yAxis
            title: {
                text: 'Donation Total',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
            labels: {
                format: '${value:,.0f}',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
            min: 0,
        },
        tooltip: {
            formatter: function () {
                return '<b>' + this.series.name + '</b><br/>' +
                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>$' +
                    Highcharts.numberFormat(this.y, 0);
            }
        },
        legend: {
            enabled: false
        },
        series: [{
            name: 'Donation Total',
            data: _.zip(timestamps, totals),
            color: Highcharts.getOptions().colors[0]
        }]
    });
};

var createRateChart = function(timestamps, rate_data) {

	Highcharts.setOptions({
        global: {
            useUTC: false
        }
    });

    return Highcharts.chart('rate-container', {
        chart: {
            type: 'spline',
            animation: Highcharts.svg,
        },
        title: {
            text: '$ Per Hour'
        },
        subtitle: {
            text: "Average, top 2.5% donations removed",
        },
        xAxis: {
            type: 'datetime',
            labels: {
                format: '{value:%a %l%P}',
            },
        },
        yAxis: { // Primary yAxis
            title: {
                text: '$ Per Hour',
                style: {
                    color: Highcharts.getOptions().colors[1]
                }
            },
            labels: {
                format: '${value:,.0f}',
                style: {
                    color: Highcharts.getOptions().colors[1]
                }
            },
            min: 0,
        },
        tooltip: {
            formatter: function () {
                return '<b>' + this.series.name + '</b><br/>' +
                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>$' +
                    Highcharts.numberFormat(this.y, 0);
            }
        },
        legend: {
            enabled: false
        },
        series: [{
            name: '$ Per Hour [Average]',
            data: _.zip(timestamps, rate_data),
            color: Highcharts.getOptions().colors[1]
        }]
    });
};

var createPerDonationChart = function(timestamps, per_donation_data) {
	Highcharts.setOptions({
        global: {
            useUTC: false
        }
    });

    return Highcharts.chart('per-donation-container', {
        chart: {
            type: 'spline',
            animation: Highcharts.svg,
        },
        title: {
            text: '$ Per Donation'
        },
        subtitle: {
            text: "Average, top 2.5% donations removed",
        },
        xAxis: {
            type: 'datetime',
            labels: {
                format: '{value:%a %l%P}',
            },
        },
        yAxis: { // Primary yAxis
            title: {
                text: '$ Per Donation [Average]',
                style: {
                    color: Highcharts.getOptions().colors[2]
                }
            },
            labels: {
                format: '${value:,.2f}',
                style: {
                    color: Highcharts.getOptions().colors[2]
                }
            },
            min:0,
        },
        tooltip: {
            formatter: function () {
                return '<b>' + this.series.name + '</b><br/>' +
                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>$' +
                    Highcharts.numberFormat(this.y, 0);
            }
        },
        legend: {
            enabled: false
        },
        series: [{
            name: '$ Per Donation [Average]',
            data: _.zip(timestamps, per_donation_data),
            color: Highcharts.getOptions().colors[2]
        }]
    });
};

//perform exponential smoothing in-place on the provided data
var exponentialSmooth = function(listOfNumbers, smoothFactor) {
	for(var i = 1; i < listOfNumbers.length; i++) {
		listOfNumbers[i] = smoothFactor * listOfNumbers[i] + (1 - smoothFactor) * listOfNumbers[i - 1];
	}
}

//replaces the largest n% of the numbers in the provided list with an average of the numbers to the left and right
//the goal is to provide a way to eliminate outliers
var ironOutTopPercent = function(listOfNumbers, topPercent) {
    var size = listOfNumbers.length;
	var topValues = topN(listOfNumbers, Math.floor(topPercent * size));
	topValues = arrayToObjectKeys(topValues);

	for(var i = 1; i < size - 1; i++) {
		if(listOfNumbers[i] in topValues) {
			listOfNumbers[i] = (listOfNumbers[i - 1] + listOfNumbers[i + 1])/2;
		}
	}

    if(listOfNumbers[size - 1] in topValues) {
        listOfNumbers[size - 1] = (listOfNumbers[size - 2] + listOfNumbers[size - 3])/2;
    }
}

//given a list of numbers and n, return the n largest values of listOfNumbers, in sorted order
var topN = function(listOfNumbers, n) {
	var values = listOfNumbers.slice(0, n);
	values.sort();

	for(var i = n; i < listOfNumbers.length; i++) {
		var insertIndex = _.sortedIndex(values, listOfNumbers[i]);
		if(insertIndex > 0) {
			values.splice(insertIndex, 0, listOfNumbers[i]);
			values.splice(0, 1);
		}
	}

	return values;
}

var arrayToObjectKeys = function(arr) {
	return arr.reduce(function(acc, cur, i) {
		acc[cur] = true;
		return acc;
	}, {});
}